use sqlx::{Error as SqlxError, Pool, Sqlite, SqlitePool};

#[cfg(test)]
pub(super) struct DBPoolFixture {
    pub(super) pool: Pool<Sqlite>,
}

#[cfg(test)]
impl DBPoolFixture {
    pub(super) async fn new() -> Self {
        let pool = get_in_memory_db_pool()
            .await
            .expect("in-memory sqlite pool should've been created");

        Self { pool }
    }
}

#[allow(unused)]
async fn get_in_memory_db_pool() -> Result<Pool<Sqlite>, SqlxError> {
    let db = SqlitePool::connect("sqlite://:memory:").await?;

    sqlx::migrate!().run(&db).await?;

    Ok(db)
}
