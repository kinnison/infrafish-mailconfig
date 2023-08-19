pub mod sql_types;
mod util;

use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, Queryable};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
pub use sql_types::MailEntryKind;

pub use self::util::Authorisation;

// These types need to match up with the schema

#[derive(Queryable)]
pub struct MailUser {
    pub id: i32,
    pub username: String,
    pub superuser: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::mailuser)]
pub struct NewMailUser<'a> {
    pub username: &'a str,
    pub superuser: bool,
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

#[derive(Queryable)]
pub struct MailDomainKey {
    pub id: i32,
    pub maildomain: i32,
    pub selector: String,
    pub privkey: String,
    pub pubkey: String,
    pub signing: bool,
}

#[derive(Insertable)]
#[diesel(table_name=crate::schema::maildomainkey)]
pub struct NewMailDomainKey<'a> {
    pub maildomain: i32,
    pub selector: &'a str,
    pub privkey: &'a str,
    pub pubkey: &'a str,
    pub signing: bool,
}

// Below here are the implementations

impl MailDomain {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        db: &mut AsyncPgConnection,
        domain_name: &str,
        owner: i32,
        remote_mx: Option<&str>,
        sender_verify: bool,
        grey_listing: bool,
        virus_check: bool,
        spamcheck_threshold: i32,
    ) -> QueryResult<Self> {
        let newdom = NewMailDomain {
            owner,
            domainname: domain_name,
            remotemx: remote_mx,
            sender_verify,
            grey_listing,
            virus_check,
            spamcheck_threshold,
        };

        use crate::schema::maildomain::dsl;

        diesel::insert_into(dsl::maildomain)
            .values(&newdom)
            .get_result(db)
            .await
    }

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
        auth: &Authorisation,
    ) -> QueryResult<bool> {
        Ok(auth.superuser() || self.owner == auth.user())
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

    pub async fn entries(&self, db: &mut AsyncPgConnection) -> QueryResult<Vec<MailEntry>> {
        use crate::schema::mailentry::dsl;

        dsl::mailentry
            .filter(dsl::maildomain.eq(self.id))
            .order_by(dsl::name.desc())
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

    pub async fn by_name(db: &mut AsyncPgConnection, name: &str) -> QueryResult<Option<Self>> {
        use crate::schema::mailuser::dsl;

        dsl::mailuser
            .filter(dsl::username.eq(name))
            .first(db)
            .await
            .optional()
    }
}

impl MailDomainKey {
    pub async fn by_domain(db: &mut AsyncPgConnection, maildomain: i32) -> QueryResult<Vec<Self>> {
        use crate::schema::maildomainkey::dsl;

        dsl::maildomainkey
            .filter(dsl::maildomain.eq(maildomain))
            .order_by(dsl::selector.asc())
            .get_results(db)
            .await
    }

    pub async fn by_domain_and_selector(
        db: &mut AsyncPgConnection,
        maildomain: i32,
        selector: &str,
    ) -> QueryResult<Option<Self>> {
        use crate::schema::maildomainkey::dsl;

        dsl::maildomainkey
            .filter(dsl::maildomain.eq(maildomain))
            .filter(dsl::selector.eq(selector))
            .first(db)
            .await
            .optional()
    }

    pub async fn save(&self, db: &mut AsyncPgConnection) -> QueryResult<()> {
        use crate::schema::maildomainkey::dsl;

        diesel::update(dsl::maildomainkey)
            .filter(dsl::id.eq(self.id))
            .set((
                dsl::selector.eq(&self.selector),
                dsl::signing.eq(self.signing),
            ))
            .execute(db)
            .await
            .map(|_| ())
    }

    pub fn render_pubkey(&self) -> String {
        format!("v=DKIM1; k=rsa; p={pubkey}", pubkey = &self.pubkey)
    }

    pub async fn create(
        db: &mut AsyncPgConnection,
        maildomain: i32,
        selector: &str,
        signing: bool,
    ) -> QueryResult<Self> {
        use crate::schema::maildomainkey::dsl;

        let (privkey, pubkey) = util::create_dkim_pair()?;

        let newkey = NewMailDomainKey {
            maildomain,
            selector,
            privkey: &privkey,
            pubkey: &pubkey,
            signing,
        };

        diesel::insert_into(dsl::maildomainkey)
            .values(newkey)
            .get_result(db)
            .await
    }

    pub async fn delete_self(self, db: &mut AsyncPgConnection) -> QueryResult<()> {
        use crate::schema::maildomainkey::dsl;

        diesel::delete(dsl::maildomainkey)
            .filter(dsl::id.eq(self.id))
            .execute(db)
            .await
            .map(|_| ())
    }
}
