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

    Ok(get_title_from_html(&body))
}

fn get_title_from_html(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

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

    title
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_simple_html_works() {
        // GIVEN
        let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>dhth/bmm: get to your bookmarks in a flash</title>
</head>
<body>
    <h1>bmm</h1>
    <p>get to your bookmarks in a flash</p>
</body>
</html>
"#;

        // WHEN
        let title = get_title_from_html(html).expect("title should've been found");

        // THEN
        assert_eq!(title, "dhth/bmm: get to your bookmarks in a flash");
    }

    #[test]
    fn parsing_html_with_og_tags_only_works() {
        // GIVEN
        let html = r#"
<title>GitHub - dhth/bmm: get to your bookmarks in a flash</title>
<meta name="description" content="get to your bookmarks in a flash. Contribute to dhth/bmm development by creating an account on GitHub.">

<meta property="og:url" content="https://github.com/dhth/bmm">
<meta property="og:type" content="website">
<meta property="og:title" content="dhth/bmm: get to your bookmarks in a flash">
<meta property="og:description" content="get to your bookmarks in a flash. Contribute to dhth/bmm development by creating an account on GitHub.">
<meta property="og:image" content="https://repository-images.githubusercontent.com/931826132/93fe31f5-18b2-4a77-9f80-9a6952ee4915">
"#;

        // WHEN
        let title = get_title_from_html(html).expect("title should've been found");

        // THEN
        assert_eq!(title, "dhth/bmm: get to your bookmarks in a flash");
    }

    #[test]
    fn parsing_html_with_both_title_and_og_tags_works() {
        // GIVEN
        let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>dhth/bmm</title>
    <meta property="og:url" content="https://github.com/dhth/bmm">
    <meta property="og:type" content="website">
    <meta property="og:title" content="dhth/bmm: get to your bookmarks in a flash">
    <meta property="og:description" content="get to your bookmarks in a flash. Contribute to dhth/bmm development by creating an account on GitHub.">
    <meta property="og:image" content="https://repository-images.githubusercontent.com/931826132/93fe31f5-18b2-4a77-9f80-9a6952ee4915">
</head>
<body>
    <h1>bmm</h1>
    <p>get to your bookmarks in a flash</p>
</body>
</html>
"#;

        // WHEN
        let title = get_title_from_html(html).expect("title should've been found");

        // THEN
        assert_eq!(title, "dhth/bmm: get to your bookmarks in a flash");
    }

    #[test]
    fn parsing_empty_html_works() {
        // GIVEN
        let html = r#"
<!DOCTYPE html>
<html lang="en">
</html>
"#;

        // WHEN
        let title = get_title_from_html(html);

        // THEN
        assert!(title.is_none());
    }

    #[test]
    fn parsing_incorrect_html_works() {
        // GIVEN
        let html = r#"
not:::html
"#;

        // WHEN
        let title = get_title_from_html(html);

        // THEN
        assert!(title.is_none());
    }
}
