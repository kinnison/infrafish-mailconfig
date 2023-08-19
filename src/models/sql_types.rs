use diesel::{
    backend::Backend, deserialize::FromSql, pg::Pg, query_builder::QueryId, serialize::ToSql,
    sql_types::Text, AsExpression, FromSqlRow, SqlType,
};

use crate::schema::sql_types::Mailentrykind as MailEntryKindType;

#[derive(Debug, FromSqlRow, AsExpression, SqlType)]
#[diesel(sql_type = MailEntryKindType)]
pub enum MailEntryKind {
    Login,
    Account,
    Alias,
    Bouncer,
    Blackhole,
}

impl<DB: Backend> ToSql<MailEntryKindType, DB> for MailEntryKind
where
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        match *self {
            MailEntryKind::Login => ("login").to_sql(out),
            MailEntryKind::Account => ("account").to_sql(out),
            MailEntryKind::Alias => ("alias").to_sql(out),
            MailEntryKind::Bouncer => ("bouncer").to_sql(out),
            MailEntryKind::Blackhole => ("blackhole").to_sql(out),
        }
    }
}

impl FromSql<MailEntryKindType, Pg> for MailEntryKind {
    fn from_sql(
        bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"login" => Ok(Self::Login),
            b"account" => Ok(Self::Account),
            b"alias" => Ok(Self::Alias),
            b"bouncer" => Ok(Self::Bouncer),
            b"blackhole" => Ok(Self::Blackhole),
            _ => Err("Unrecognised mail entry kind variant".into()),
        }
    }
}

impl QueryId for crate::schema::sql_types::Mailentrykind {
    type QueryId = Self;

    const HAS_STATIC_QUERY_ID: bool = true;
}
