use crate::persistence::SearchTerms;

#[derive(Clone, Debug)]
pub(super) enum Command {
    OpenInBrowser(String),
    SearchBookmarks(SearchTerms),
    FetchTags,
    FetchBookmarksForTag(String),
}
