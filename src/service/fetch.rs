use reqwest::Error as ReqwestError;
use reqwest::get;
use scraper::{Html, Selector};
use url::{ParseError, Url};

#[derive(thiserror::Error, Debug)]
pub enum FetchUriDetailsError {
    #[error("uri is incorrect: {0}")]
    IncorrectUri(#[from] ParseError),
    #[error(transparent)]
    RequestUri(#[from] ReqwestError),
}

pub async fn fetch_uri_details(uri: &str) -> Result<Option<String>, FetchUriDetailsError> {
    Url::parse(uri)?;
    let body = get(uri).await?.text().await?;

    let document = Html::parse_document(&body);

    #[allow(clippy::unwrap_used)]
    let title_selector = Selector::parse("title").unwrap();

    #[allow(clippy::unwrap_used)]
    let og_title_selector = Selector::parse(r#"meta[property="og:title"]"#).unwrap();

    let og_title = document
        .select(&og_title_selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(|s| s.trim().to_string());

    let title = og_title.or_else(|| {
        document
            .select(&title_selector)
            .next()
            .map(|element| element.inner_html().trim().to_string())
    });

    Ok(title)
}
