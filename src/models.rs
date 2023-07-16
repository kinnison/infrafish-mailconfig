pub mod sql_types;

use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, Queryable};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
pub use sql_types::MailEntryKind;

// These types need to match up with the schema

#[derive(Queryable)]
pub struct MailUser {
    pub id: i32,
    pub username: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::mailuser)]
pub struct NewMailUser<'a> {
    pub username: &'a str,
}

#[derive(Queryable)]
pub struct MailDomain {
    pub id: i32,
    pub owner: i32,
    pub domainname: String,
    pub remotemx: Option<String>,
    pub sender_verify: bool,
    pub grey_listing: bool,
    pub virus_check: bool,
    pub spamcheck_threshold: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::maildomain)]
pub struct NewMailDomain<'a> {
    pub owner: i32,
    pub domainname: &'a str,
    pub remotemx: Option<&'a str>,
    pub sender_verify: bool,
    pub grey_listing: bool,
    pub virus_check: bool,
    pub spamcheck_threshold: i32,
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
    pub maildomain: i32,
    pub name: &'a str,
    pub kind: MailEntryKind,
    pub password: Option<&'a str>,
    pub expansion: Option<&'a str>,
}

#[derive(Queryable)]
pub struct AllowDenyList {
    pub id: i32,
    pub maildomain: i32,
    pub allow: bool,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name=crate::schema::allowdenylist)]
pub struct NewAllowDenyList<'a> {
    pub maildomain: i32,
    pub allow: bool,
    pub value: &'a str,
}

#[derive(Queryable)]
pub struct MailAuthToken {
    pub id: i32,
    pub mailuser: i32,
    pub token: String,
    pub label: String,
}

#[derive(Insertable)]
#[diesel(table_name=crate::schema::mailauthtoken)]
pub struct NewMailAuthToken<'a> {
    pub mailuser: i32,
    pub token: &'a str,
    pub label: &'a str,
}

// Below here are the implementations

impl MailDomain {
    pub async fn get_all(db: &mut AsyncPgConnection) -> QueryResult<Vec<Self>> {
        use crate::schema::maildomain::dsl;
        dsl::maildomain
            .order_by(dsl::domainname.asc())
            .get_results(db)
            .await
    }
}

impl AllowDenyList {
    pub async fn all_allows(
        db: &mut AsyncPgConnection,
        maildomain: i32,
    ) -> QueryResult<Vec<String>> {
        use crate::schema::allowdenylist::dsl;

        dsl::allowdenylist
            .filter(dsl::maildomain.eq(maildomain))
            .filter(dsl::allow.eq(true))
            .order_by(dsl::value.asc())
            .get_results(db)
            .await
            .map(|v| v.into_iter().map(|adv: Self| adv.value).collect())
    }

    pub async fn all_denys(
        db: &mut AsyncPgConnection,
        maildomain: i32,
    ) -> QueryResult<Vec<String>> {
        use crate::schema::allowdenylist::dsl;

        dsl::allowdenylist
            .filter(dsl::maildomain.eq(maildomain))
            .filter(dsl::allow.eq(false))
            .order_by(dsl::value.asc())
            .get_results(db)
            .await
            .map(|v| v.into_iter().map(|adv: Self| adv.value).collect())
    }
}
