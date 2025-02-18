use super::commands::Command;
use super::message::{Message, UrlsOpenedResult};
use crate::common::DEFAULT_LIMIT;
use crate::persistence::{get_bookmarks, get_bookmarks_by_query, get_tags_with_stats};
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc::Sender;

pub(super) async fn handle_command(
    pool: &Pool<Sqlite>,
    command: Command,
    event_tx: Sender<Message>,
) {
    match command {
        // TODO: handle errors here
        Command::OpenInBrowser(url) => {
            tokio::spawn(async move {
                let message = match open::that(url) {
                    Ok(_) => Message::UrlsOpenedInBrowser(UrlsOpenedResult::Success),
                    Err(e) => Message::UrlsOpenedInBrowser(UrlsOpenedResult::Failure(e)),
                };

                let _ = event_tx.try_send(message);
            });
        }
        Command::SearchBookmarks(search_query) => {
            let pool = pool.clone();
            tokio::spawn(async move {
                let result = get_bookmarks_by_query(&pool, &search_query, DEFAULT_LIMIT).await;
                let message = Message::SearchFinished(result);
                let _ = event_tx.try_send(message);
            });
        }
        Command::FetchTags => {
            let pool = pool.clone();
            tokio::spawn(async move {
                let result = get_tags_with_stats(&pool).await;
                let message = Message::TagsFetched(result);
                let _ = event_tx.try_send(message);
            });
        }
        Command::FetchBookmarksForTag(tag) => {
            let pool = pool.clone();
            tokio::spawn(async move {
                let result = get_bookmarks(&pool, None, None, vec![tag], DEFAULT_LIMIT).await;
                let message = Message::BookmarksForTagFetched(result);
                let _ = event_tx.try_send(message);
            });
        }
    }
}
