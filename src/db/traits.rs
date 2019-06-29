//! Traits for query-building convenience.

use db::connection::Connection;
use error::GreaseResult;
use pinto::query_builder::*;

/// Consistent way to provide a table name for a query.
pub trait TableName {
    /// Returns the model's related table name.
    fn table_name() -> &'static str;
}

/// Consistent way to get the fields of a model in order.
pub trait FieldNames {
    /// Returns a slice of field names.
    fn field_names() -> &'static [&'static str];
}

/// Allows method-style inserts of models.
pub trait Insertable: TableName + Sized {
    /// Just inserts a model as a row.
    fn insert<C: Connection>(&self, conn: &mut C) -> GreaseResult<()>;

    /// Inserts multiple rows into a table.
    fn insert_multiple<C: Connection>(to_insert: &[Self], conn: &mut C) -> GreaseResult<()>;

    /// Inserts a model as a row, and then returns the new row's id.
    ///
    /// This is intended only for models with "id" primary keys, so use
    /// this carefully.
    fn insert_returning_id<C: Connection>(&self, conn: &mut C) -> GreaseResult<i32>;
}

/// Convenient methods to select rows from a model's related table.
pub trait Selectable: TableName + FieldNames + mysql::prelude::FromRow + Sized {
    /// Basically `SELECT * FROM <table name> WHERE <filter>`
    fn filter<'a>(filter: &'a str) -> Select<'a> {
        let mut query = Select::new(Self::table_name());
        query.fields(Self::field_names()).filter(filter);

        query
    }

    /// Basically `SELECT * FROM <table name>`
    fn select_all() -> Select<'static> {
        let mut query = Select::new(Self::table_name());
        query.fields(Self::field_names());

        query
    }

    /// Basically `SELECT * FROM <table name> ORDER BY <field in direction>`
    fn select_all_in_order<'a>(field_name: &'a str, direction: Order) -> Select<'a> {
        let mut query = Select::new(Self::table_name());
        query
            .fields(Self::field_names())
            .order_by(field_name, direction);

        query
    }
}

impl<T: TableName + FieldNames + mysql::prelude::FromRow + Sized> Selectable for T {}
