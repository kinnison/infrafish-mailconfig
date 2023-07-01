use std::{fmt::Display, time::Duration};

use bb8::ErrorSink;
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, PoolError},
    AsyncPgConnection,
};
use futures::{future::BoxFuture, FutureExt};
use lazy_static::lazy_static;
use rustls::RootCertStore;
use tokio_postgres_rustls::MakeRustlsConnect;
use tracing::error;

mod schema;

pub mod models;

pub fn apply_migrations(db_url: &str) -> diesel::migration::Result<()> {
    use diesel::{Connection, PgConnection};
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    let mut conn = PgConnection::establish(db_url)?;
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub type Pool = diesel_async::pooled_connection::bb8::Pool<AsyncPgConnection>;

lazy_static! {
    static ref MAKE_TLS_CONNECT: MakeRustlsConnect = {
        let mut store = RootCertStore::empty();
        store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));
        MakeRustlsConnect::new(
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(store)
                .with_no_client_auth(),
        )
    };
}

fn establish_connection(url: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    (async {
        let (client, connection) = tokio_postgres::connect(url, MAKE_TLS_CONNECT.clone())
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    })
    .boxed()
}

#[derive(Debug)]
struct MyErrorSink;

impl<E> ErrorSink<E> for MyErrorSink
where
    E: Display,
{
    fn sink(&self, err: E) {
        error!("BB8 pool error: {err}");
    }

    fn boxed_clone(&self) -> Box<dyn ErrorSink<E>> {
        Box::new(MyErrorSink)
    }
}

pub async fn create_pool(db_url: &str) -> Result<Pool, PoolError> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(
        db_url,
        establish_connection,
    );
    bb8::Pool::builder()
        .idle_timeout(Duration::from_secs(30).into())
        .connection_timeout(Duration::from_secs(10))
        .error_sink(Box::new(MyErrorSink))
        .build(config)
        .await
}
