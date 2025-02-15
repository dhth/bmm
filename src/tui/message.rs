use super::common::View;

#[derive(PartialEq)]
pub(crate) enum Message {
    TerminalResize(w, h),
    GoToNextItem,
    GoToPreviousPreview,
    GoToFirstItem,
    GoToLastItem,
    OpenInBrowser,
    ShowView(View),
    Quit,
}
