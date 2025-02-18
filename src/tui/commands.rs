#[derive(Clone, Debug)]
pub(super) enum Command {
    OpenInBrowser(String),
    SearchBookmarks(String),
    FetchTags,
    FetchBookmarksForTag(String),
}
