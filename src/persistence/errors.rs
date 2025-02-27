use sqlx::Error as SqlxError;
use sqlx::migrate::MigrateError;

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("couldn't check if db exists: {0}")]
    CouldntCheckIfDbExists(#[source] SqlxError),
    #[error("couldn't create database: {0}")]
    CouldntCreateDatabase(#[source] SqlxError),
    #[error("couldn't connect to database: {0}")]
    CouldntConnectToDB(#[source] SqlxError),
    #[error("couldn't migrate database: {0}")]
    CouldntMigrateDB(#[source] MigrateError),
    #[error("couldn't execute query ({0}): {1}")]
    CouldntExecuteQuery(String, #[source] SqlxError),
    #[error("couldn't convert from sql: {0}")]
    CouldntConvertFromSQL(#[source] SqlxError),
    #[error("couldn't begin transation: {0}")]
    CouldntBeginTransaction(#[source] SqlxError),
    #[error("couldn't commit transation: {0}")]
    CouldntCommitTransaction(#[source] SqlxError),
}
