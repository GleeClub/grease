use error::{GreaseError, GreaseResult};
use mysql::prelude::{FromValue, GenericConnection};
use mysql::Conn;
use pinto::query_builder::{self, Order, Select};

pub trait TableName {
    fn table_name() -> &'static str;
}

pub trait FieldNames {
    fn field_names() -> &'static [&'static str];
}

pub trait Insertable: TableName {
    fn insert<G: GenericConnection>(&self, conn: &mut G) -> GreaseResult<()>;

    fn insert_returning_id<T: FromValue>(
        &self,
        id_field: &str,
        conn: &mut Conn,
    ) -> GreaseResult<T> {
        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;

        self.insert(&mut transaction)?;
        let query_id = Select::new(Self::table_name())
            .fields(&[id_field])
            .order_by(id_field, Order::Desc)
            .build();
        let id = match transaction.first(query_id) {
            Ok(Some(id)) => Ok(id),
            Ok(None) => Err(GreaseError::ServerError(format!(
                "error inserting row into table {}",
                Self::table_name()
            ))),
            Err(error) => Err(GreaseError::DbError(error)),
        }?;
        transaction.commit().map_err(GreaseError::DbError)?;

        Ok(id)
    }
}

pub trait Queryable: TableName + FieldNames + mysql::prelude::FromRow + Sized {
    fn first_opt<G: GenericConnection>(filter: &str, conn: &mut G) -> GreaseResult<Option<Self>> {
        let query = Select::new(Self::table_name())
            .fields(Self::field_names())
            .filter(filter)
            .build();

        match conn.first(query) {
            Ok(maybe_returned) => Ok(maybe_returned),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    fn first<G: GenericConnection>(
        filter: &str,
        conn: &mut G,
        missing_message: String,
    ) -> GreaseResult<Self> {
        Self::first_opt(filter, conn).and_then(|maybe_returned| {
            maybe_returned.ok_or(GreaseError::BadRequest(missing_message))
        })
    }

    fn query_all<G: GenericConnection>(conn: &mut G) -> GreaseResult<Vec<Self>> {
        crate::db::load(
            &Select::new(Self::table_name())
                .fields(Self::field_names())
                .build(),
            conn,
        )
    }

    fn query_all_in_order<G: GenericConnection>(
        orders: Vec<(&'static str, Order)>,
        conn: &mut G,
    ) -> GreaseResult<Vec<Self>> {
        let mut query = query_builder::select(Self::table_name());
        query.fields(Self::field_names());
        for (field, direction) in orders {
            query.order_by(field, direction);
        }

        crate::db::load(&query.build(), conn)
    }
}

impl<T: TableName + FieldNames + mysql::prelude::FromRow + Sized> Queryable for T {}
