pub mod sql_types;

use diesel::dsl::sql;
use diesel::prelude::*;
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

    pub async fn by_owner(db: &mut AsyncPgConnection, owner: i32) -> QueryResult<Vec<Self>> {
        use crate::schema::maildomain::dsl;

        dsl::maildomain
            .filter(dsl::owner.eq(owner))
            .order_by(dsl::domainname.asc())
            .get_results(db)
            .await
    }

    pub async fn by_name(db: &mut AsyncPgConnection, name: &str) -> QueryResult<Option<Self>> {
        use crate::schema::maildomain::dsl;

        dsl::maildomain
            .filter(dsl::domainname.eq(name))
            .get_result(db)
            .await
            .optional()
    }

    pub async fn may_access(
        &self,
        _db: &mut AsyncPgConnection,
        authuser: i32,
    ) -> QueryResult<bool> {
        Ok(self.owner == authuser)
    }

    pub async fn save(&self, db: &mut AsyncPgConnection) -> QueryResult<()> {
        use crate::schema::maildomain::dsl;

        diesel::update(dsl::maildomain)
            .filter(dsl::id.eq(self.id))
            .set((
                dsl::domainname.eq(&self.domainname),
                dsl::remotemx.eq(self.remotemx.as_deref()),
                dsl::grey_listing.eq(self.grey_listing),
                dsl::sender_verify.eq(self.sender_verify),
                dsl::spamcheck_threshold.eq(self.spamcheck_threshold),
                dsl::virus_check.eq(self.virus_check),
            ))
            .execute(db)
            .await
            .map(|_| ())
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

impl MailAuthToken {
    pub async fn by_token(db: &mut AsyncPgConnection, token: &str) -> QueryResult<Option<Self>> {
        use crate::schema::mailauthtoken::dsl;

        dsl::mailauthtoken
            .filter(dsl::token.eq(token))
            .first(db)
            .await
            .optional()
    }

    pub async fn by_owner(db: &mut AsyncPgConnection, owner: i32) -> QueryResult<Vec<Self>> {
        use crate::schema::mailauthtoken::dsl;

        dsl::mailauthtoken
            .filter(dsl::mailuser.eq(owner))
            .get_results(db)
            .await
    }

    pub async fn create(db: &mut AsyncPgConnection, owner: i32, label: &str) -> QueryResult<Self> {
        use crate::schema::mailauthtoken::dsl;

        diesel::insert_into(dsl::mailauthtoken)
            .values((
                dsl::mailuser.eq(owner),
                dsl::label.eq(label),
                dsl::token.eq(sql("md5(gen_random_uuid()::varchar)")),
            ))
            .get_result(db)
            .await
    }

    pub async fn delete_self(self, db: &mut AsyncPgConnection) -> QueryResult<()> {
        use crate::schema::mailauthtoken::dsl;

        diesel::delete(dsl::mailauthtoken)
            .filter(dsl::id.eq(self.id))
            .execute(db)
            .await
            .map(|_| ())
    }
}

impl MailUser {
    pub async fn by_id(db: &mut AsyncPgConnection, id: i32) -> QueryResult<Self> {
        use crate::schema::mailuser::dsl;
        dsl::mailuser.filter(dsl::id.eq(id)).first(db).await
    }
}
