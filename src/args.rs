use crate::common::IMPORT_FILE_FORMATS;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

const NOT_PROVIDED: &str = "<not provided>";

/// bmm lets you manage your bookmarks via the command line
#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    #[command(subcommand)]
    pub command: BmmCommand,
    /// override bmm's database location (default: <DATA_DIR>/bmm/bmm.db)
    #[arg(long = "db-path", value_name = "STRING", global = true)]
    pub db_path: Option<String>,
    /// output debug information without doing anything
    #[arg(long = "debug", global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum BmmCommand {
    /// Import bookmarks from various sources
    Import {
        /// File to import from; the file's extension will be used to infer file format; supported formats: [html, json, txt, markdown]
        #[arg(value_name = "FILE", value_parser=validate_import_file)]
        file: String,
        /// Display bookmarks that will be imported without actually importing them
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,
    },
    /// List bookmarks based on several kinds of queries
    List {
        /// Query to match bookmark uris on
        #[arg(short = 'u', long = "uri", value_name = "URI")]
        uri: Option<String>,
        /// Query to match bookmark titles on
        #[arg(short = 'd', long = "title", value_name = "STRING")]
        title: Option<String>,
        /// Query to match bookmark tags on
        #[arg(
            short = 't',
            long = "tags",
            value_name = "STRING,STRING..",
            value_delimiter = ','
        )]
        tags: Vec<String>,
        /// Format to use
        #[arg(
            short = 'f',
            long = "format",
            value_name = "STRING",
            default_value = "plain"
        )]
        format: OutputFormat,
        /// Number of items to fetch
        #[arg(
            short = 'l',
            long = "limit",
            value_name = "INTEGER",
            default_value_t = 500
        )]
        limit: u16,
    },
    /// Saves a bookmark.
    Save {
        /// Uri of the bookmark
        #[arg(value_name = "URI")]
        uri: String,
        /// Title for the bookmark
        #[arg(long = "title", value_name = "STRING")]
        title: Option<String>,
        /// Tags to attach to the bookmark
        #[arg(
            short = 't',
            long = "tags",
            value_name = "STRING,STRING..",
            value_delimiter = ','
        )]
        tags: Option<Vec<String>>,
        /// Provide details via a text editor
        #[arg(short = 'e', long = "editor")]
        use_editor: bool,
        /// Fail if uri already saved (bmm will update the existing entry by default)
        #[arg(short = 'f', long = "fail-if-already-saved", value_name = "STRING")]
        fail_if_uri_already_saved: bool,
    },
    /// Search bookmarks based on a singular query
    Search {
        /// Query to match bookmarks where any attribute of the bookmark (uri, title, tags) matches the query
        #[arg(value_name = "QUERY")]
        query: String,
        /// Format to output in
        #[arg(
            short = 'f',
            long = "format",
            value_name = "STRING",
            default_value = "plain"
        )]
        format: OutputFormat,
        /// Number of items to fetch
        #[arg(
            short = 'l',
            long = "limit",
            value_name = "INTEGER",
            default_value_t = 500
        )]
        limit: u16,
    },
    /// Show bookmark details
    Show {
        /// Query to match bookmarks where any attribute of the bookmark (uri, title, tags) matches the query
        #[arg(value_name = "URI")]
        uri: String,
    },
    /// Interact with bmm tags
    Tags {
        #[command(subcommand)]
        tags_command: TagsCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagsCommand {
    /// List tags stored by bmm
    List {
        /// Format to output in
        #[arg(
            short = 'f',
            long = "format",
            value_name = "STRING",
            default_value = "plain"
        )]
        format: OutputFormat,
        /// whether to show tag stats
        #[arg(short = 's', long = "show-stats")]
        show_stats: bool,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Delimited output
    Delimited,
    /// JSON output
    Json,
    /// Plain output
    Plain,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            OutputFormat::Plain => "plain",
            OutputFormat::Json => "json",
            OutputFormat::Delimited => "delimited",
        };

        write!(f, "{}", value)?;

        Ok(())
    }
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match &self.command {
            BmmCommand::List {
                uri,
                title,
                tags,
                format,
                limit,
            } => format!(
                r#"
command           : List bookmark(s)
uri query         : {}
title query       : {}
tags              : {:?}
format            : {}
limit             : {}
"#,
                uri.as_deref().unwrap_or(NOT_PROVIDED),
                title.as_deref().unwrap_or(NOT_PROVIDED),
                tags,
                format,
                limit
            ),
            BmmCommand::Import { file, dry_run } => format!(
                r#"
command : Import bookmarks
file    : {}
dry_run : {}
"#,
                file, dry_run
            ),
            BmmCommand::Save {
                uri,
                title,
                tags,
                use_editor,
                fail_if_uri_already_saved: fail_if_uri_saved,
            } => format!(
                r#"
command                   : Save bookmark
uri                       : {}
title                     : {}
tags                      : {}
use editor                : {}
fail if uri already saved : {}
"#,
                uri,
                title.as_deref().unwrap_or(NOT_PROVIDED),
                tags.as_ref().map_or(NOT_PROVIDED.into(), |t| t.join(" ")),
                use_editor,
                fail_if_uri_saved,
            ),
            BmmCommand::Search {
                query: uri,
                format,
                limit,
            } => format!(
                r#"
command     : Search bookmarks
query       : {}
format      : {}
limit       : {}
"#,
                uri, format, limit
            ),
            BmmCommand::Show { uri } => format!(
                r#"
command     : Show bookmarks
uri         : {}
"#,
                uri
            ),
            BmmCommand::Tags { tags_command } => match tags_command {
                TagsCommand::List { format, show_stats } => format!(
                    r#"
command      : List Tags
format       : {:?}
show stats   : {}
"#,
                    format, show_stats
                ),
            },
        };

        f.write_str(&output)
    }
}

fn validate_import_file(file: &str) -> Result<String, String> {
    let path_buf = PathBuf::from(file);
    match path_buf.extension() {
        Some(e) => match e.to_str() {
            Some(ext) => {
                if !IMPORT_FILE_FORMATS.contains(&ext) {
                    return Err(format!(
                        "only the following file formats are supported for import: {:?}",
                        IMPORT_FILE_FORMATS
                    ));
                }
            }
            None => {
                return Err(format!(
                    "file has invalid extension; supported extensions: {:?}",
                    IMPORT_FILE_FORMATS
                ));
            }
        },
        None => {
            return Err(format!(
                "file has no extension; supported extensions: {:?}",
                IMPORT_FILE_FORMATS
            ));
        }
    }

    Ok(file.to_string())
}
