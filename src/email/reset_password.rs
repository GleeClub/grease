use askama::Template;
use lettre::message::Mailbox;

use crate::email::Email;
use crate::models::member::Member;

#[derive(Template)]
#[template(path = "reset-password.html")]
pub struct ResetPasswordEmail<'a> {
    pub member: &'a Member,
    pub token: &'a str,
}

impl<'a> Email for ResetPasswordEmail<'a> {
    fn subject(&self) -> String {
        "Reset Your GlubHub Password".to_owned()
    }

    fn address(&self) -> Mailbox {
        Mailbox {
            name: Some(self.member.full_name_inner()),
            email: self.member.email.to_owned().parse().unwrap(),
        }
    }
}
