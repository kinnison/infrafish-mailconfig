pub mod sql_types;

use diesel::{Insertable, Queryable};
pub use sql_types::MailEntryKind;

#[derive(Queryable)]
pub struct MailUser {
    pub id: i32,
    pub username: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::mailuser)]
pub struct NewMailUser<'a> {
    pub id: i32,
    pub username: &'a str,
}

#[derive(Queryable)]
pub struct MailDomain {
    pub id: i32,
    pub owner: i32,
    pub domainname: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::maildomain)]
pub struct NewMailDomain<'a> {
    pub id: i32,
    pub owner: i32,
    pub domainname: &'a str,
}

#[derive(Queryable)]
pub struct MailEntry {
    pub id: i32,
    pub maildomain: i32,
    pub name: String,
    pub kind: MailEntryKind,
    pub password: Option<String>,
    pub expansion: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name=crate::schema::mailentry)]
pub struct NewMailEntry<'a> {
    pub id: i32,
    pub maildomain: i32,
    pub name: &'a str,
    pub kind: MailEntryKind,
    pub password: Option<&'a str>,
    pub expansion: Option<&'a str>,
}
