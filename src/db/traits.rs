use db::connection::Connection;
use error::GreaseResult;
use pinto::query_builder::*;

pub trait TableName {
    fn table_name() -> &'static str;
}

pub trait FieldNames {
    fn field_names() -> &'static [&'static str];
}

pub trait Insertable: TableName + Sized {
    fn insert<C: Connection>(&self, conn: &mut C) -> GreaseResult<()>;

    fn insert_multiple<C: Connection>(to_insert: &[Self], conn: &mut C) -> GreaseResult<()>;

    fn insert_returning_id<C: Connection>(&self, conn: &mut C) -> GreaseResult<i32>;
}

pub trait Selectable: TableName + FieldNames + mysql::prelude::FromRow + Sized {
    fn filter<'a>(filter: &'a str) -> Select<'a> {
        let mut query = Select::new(Self::table_name());
        query.fields(Self::field_names()).filter(filter);

        query
    }

    fn select_all() -> Select<'static> {
        let mut query = Select::new(Self::table_name());
        query.fields(Self::field_names());

        query
    }

    fn select_all_in_order<'a>(field_name: &'a str, direction: Order) -> Select<'a> {
        let mut query = Select::new(Self::table_name());
        query
            .fields(Self::field_names())
            .order_by(field_name, direction);

        query
    }
}

impl<T: TableName + FieldNames + mysql::prelude::FromRow + Sized> Selectable for T {}
