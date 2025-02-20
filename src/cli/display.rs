use crate::args::Args;
use crate::args::OutputFormat;
use crate::domain::{SavedBookmark, TagStats};
use csv::Error as CsvError;
use serde_json::Error as SerdeJsonError;

const NOT_SET: &str = "<NOT SET>";

#[derive(thiserror::Error, Debug)]
pub enum DisplayError {
    #[error("couldn't serialize response to JSON: {0}")]
    CouldntSerializeToJSON(#[from] SerdeJsonError),
    #[error("couldn't serialize response to CSV: {0}")]
    CouldntSerializeToCSV(#[from] CsvError),
    #[error("couldn't flush contents to csv writer: {0}")]
    CouldntFlushResultsToCSVWriter(#[from] std::io::Error),
}

pub fn display_bookmarks(
    bookmarks: &Vec<SavedBookmark>,
    format: &OutputFormat,
) -> Result<(), DisplayError> {
    match format {
        OutputFormat::Plain => {
            for b in bookmarks {
                println!("{}", b.uri);
            }
        }
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(&bookmarks)
                .map_err(DisplayError::CouldntSerializeToJSON)?;
            println!("{}", output);
        }
        OutputFormat::Delimited => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            for b in bookmarks {
                wtr.serialize(b)?;
            }
            wtr.flush()?;
        }
    }

    Ok(())
}

pub fn display_bookmark_details(bookmark: &SavedBookmark) {
    println!(
        r#"Bookmark details
---

Title: {}
URI  : {}
Tags : {}"#,
        bookmark.title.as_deref().unwrap_or(NOT_SET),
        bookmark.uri,
        bookmark.tags.as_deref().unwrap_or(NOT_SET),
    )
}

pub fn display_tags(tags: &Vec<String>, format: &OutputFormat) -> Result<(), DisplayError> {
    match format {
        OutputFormat::Plain => {
            println!("{}", tags.join("\n"))
        }
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(tags)?;
            println!("{}", output);
        }
        OutputFormat::Delimited => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            for t in tags {
                wtr.serialize(t)?;
            }
            wtr.flush()?;
        }
    }

    Ok(())
}

pub fn display_tags_with_stats(
    tags: &Vec<TagStats>,
    format: &OutputFormat,
) -> Result<(), DisplayError> {
    match format {
        OutputFormat::Plain => {
            for t in tags {
                println!("{}", t);
            }
        }
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(tags)?;
            println!("{}", output);
        }
        OutputFormat::Delimited => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            for t in tags {
                wtr.serialize(t)?;
            }
            wtr.flush()?;
        }
    }

    Ok(())
}

pub fn display_debug_info(args: &Args, db_path: &str) {
    println!(
        r#"DEBUG INFO:

<your arguments>{}
<computed config>
db path: {}
"#,
        args, db_path
    )
}
