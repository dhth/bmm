#[derive(Clone, Debug)]
pub(super) enum Command {
    OpenInBrowser(String),
    SearchBookmarks(String),
}
