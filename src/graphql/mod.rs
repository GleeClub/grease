use async_graphql::{EmptySubscription, Schema};

use crate::graphql::mutation::MutationRoot;
use crate::graphql::query::QueryRoot;

pub mod guards;
pub mod mutation;
pub mod query;

pub const SUCCESS_MESSAGE: &str = "success";

pub fn build_schema() -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription)
}
