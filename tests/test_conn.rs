extern crate mysql;
extern crate serde_json;

use mysql::{Row, Value as MysqlValue};
use mysql::prelude::GenericConnection;
use serde_json::Value as JsonValue;

use mysql::prelude::GenericConnection;

pub struct TestConn {
    called: usize,
    callback: Box<Fn(&mut Self) -> MysqlValue>,
    value: JsonValue,
}

impl TestConn {
    fn new<C: Fn(&mut Self) -> MysqlValue>(callback: C, value: JsonValue) -> Self {
        TestConn {
            called: 0,
            callback: Box::new(callback),
            value,
        }
    }

    fn json_to_row(json_val: &JsonValue) -> Row {

    }

    fn json_to_mysql(json_val: &JsonValue) -> MysqlValue {
        let match json_val {

        }
            NULL,
    Bytes(Vec<u8>),
    Int(i64),
    UInt(u64),
    Float(f64),
    /// year, month, day, hour, minutes, seconds, micro seconds
    Date(u16, u8, u8, u8, u8, u8, u32),
    /// is negative, days, hours, minutes, seconds, micro seconds
    Time(bool, u32, u8, u8, u8, u32),
    }
}

impl GenericConnection for TestConn {

}