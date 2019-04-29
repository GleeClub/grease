use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use r2d2::{Config, GetTimeout, Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

pub struct DB(PooledConnection<ConnectionManager<MysqlConnection>>);

impl Deref for DB {
    type Target = &MysqlConnection;
    pub fn deref(&self) -> Self::Target {
        &*self.0
    }
}

pub fn create_db_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    let config = Config::default();
    let manager = ConnectionManager::<MysqlConnection>::new(format!("{}", DB_CREDENTIALS));
    Pool::new(config, manager).expect("Failed to create pool.")
}

lazy_static! {
    pub static ref POOL: Pool<ConnectionManager<MysqlConnection>> = create_db_pool();
}

pub struct RequestInfo<'r> {
	pub choir: String,
	pub semester: i32,
	pub user: String,
	pub conn: &'r MysqlConnection,
}
