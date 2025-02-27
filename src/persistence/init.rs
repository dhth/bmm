use super::DBError;
use sqlx::{Pool, Sqlite, SqlitePool, migrate::MigrateDatabase};

pub async fn get_db_pool(uri: &str) -> Result<Pool<Sqlite>, DBError> {
    let db_exists = Sqlite::database_exists(uri)
        .await
        .map_err(DBError::CouldntCheckIfDbExists)?;

    if !db_exists {
        Sqlite::create_database(uri)
            .await
            .map_err(DBError::CouldntCreateDatabase)?;
    }

    let db = SqlitePool::connect(uri)
        .await
        .map_err(DBError::CouldntConnectToDB)?;

    sqlx::migrate!()
        .run(&db)
        .await
        .map_err(DBError::CouldntMigrateDB)?;

    Ok(db)
}

#[cfg(test)]
pub(super) async fn get_in_memory_db_pool() -> Result<Pool<Sqlite>, DBError> {
    let db = SqlitePool::connect("sqlite://:memory:")
        .await
        .map_err(DBError::CouldntConnectToDB)?;

    sqlx::migrate!()
        .run(&db)
        .await
        .map_err(DBError::CouldntMigrateDB)?;

    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrating_db_works() {
        // GIVEN
        // WHEN
        let result = get_in_memory_db_pool().await;

        // THEN
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_conn_fails_if_path_doesnt_exist() {
        // GIVEN
        let path = "nonexistent/nonexistent/nonexistent.db";

        // WHEN
        let result = get_db_pool(path).await;

        // THEN
        assert!(result.is_err());
    }
}
