use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TagStats {
    pub name: String,
    pub num_bookmarks: i64,
}

impl std::fmt::Display for TagStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} bookmarks)", self.name, self.num_bookmarks)?;

        Ok(())
    }
}
