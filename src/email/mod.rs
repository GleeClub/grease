//! Extra utilties for use elsewhere in the API.

pub mod event;
pub mod minutes;

pub const MEMBER_LIST_ADDRESS: &str = "gleeclub@lists.gatech.edu";
pub const OFFICER_LIST_ADDRESS: &str = "gleeclub@lists.gatech.edu";
pub const FROM_ADDRESS: &str = "Glee Club Officers";

pub struct Email<'a> {
    pub address: &'a str,
    pub subject: String,
    pub body: String,
}
