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
    use insta::assert_debug_snapshot;

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
        let error = get_db_pool(path)
            .await
            .expect_err("result should've been an error");

        // THEN
        assert_debug_snapshot!(error, @r#"
        CouldntCreateDatabase(
            Database(
                SqliteError {
                    code: 14,
                    message: "unable to open database file",
                },
            ),
        )
        "#);
    }
}
