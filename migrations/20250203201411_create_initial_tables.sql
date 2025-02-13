CREATE TABLE IF NOT EXISTS bookmarks (
    id INTEGER PRIMARY KEY NOT NULL,
    uri TEXT NOT NULL UNIQUE,
    title TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_uri ON bookmarks (uri);

-- 

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_name ON tags (name);

-- 

CREATE TABLE IF NOT EXISTS bookmark_tags (
    bookmark_id INTEGER,
    tag_id INTEGER,
    PRIMARY KEY (bookmark_id, tag_id),
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_bookmark_id ON bookmark_tags (bookmark_id);
CREATE INDEX IF NOT EXISTS idx_tag_id ON bookmark_tags (tag_id);
