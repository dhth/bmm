use crate::common::IMPORT_FILE_FORMATS;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

const NOT_PROVIDED: &str = "<not provided>";
const LONG_ABOUT: &str = include_str!("static/long-about.txt");
const IMPORT_HELP: &str = include_str!("static/import-help.txt");

/// bmm lets you get to your bookmarks in a flash
#[derive(Parser, Debug)]
#[command(long_about = LONG_ABOUT.trim())]
pub struct Args {
    #[command(subcommand)]
    pub command: BmmCommand,
    /// Override bmm's database location (default: <DATA_DIR>/bmm/bmm.db)
    #[arg(long = "db-path", value_name = "STRING", global = true)]
    pub db_path: Option<String>,
    /// Output debug information without doing anything
    #[arg(long = "debug", global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum BmmCommand {
    /// Import bookmarks from various sources
    #[command(long_about = IMPORT_HELP.trim())]
    Import {
        #[arg(value_name = "FILE", value_parser=validate_import_file)]
        #[arg(help = format!("File to import from; the file's extension will be used to infer file format; supported formats: {:?}", IMPORT_FILE_FORMATS))]
        file: String,
        /// Display bookmarks that will be imported without actually importing them
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,
        /// Reset previously saved details if not provided
        #[arg(short = 'r', long = "reset-missing-details")]
        reset_missing: bool,
    },
    /// Delete bookmarks
    Delete {
        /// URIs to delete (will be matched exactly)
        #[arg(value_name = "URI")]
        uris: Vec<String>,
        /// Whether to skip confirmation
        #[arg(short = 'y', long = "yes")]
        skip_confirmation: bool,
    },
    /// List bookmarks based on several kinds of queries
    List {
        /// Pattern to match bookmark URIs on
        #[arg(short = 'u', long = "uri", value_name = "URI")]
        uri: Option<String>,
        /// Pattern to match bookmark titles on
        #[arg(short = 'd', long = "title", value_name = "STRING")]
        title: Option<String>,
        /// Tags to match (exactly)
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
    /// Save/update a bookmark
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
        /// Fail if URI already saved (bmm will update the existing entry by default)
        #[arg(short = 'f', long = "fail-if-already-saved", value_name = "STRING")]
        fail_if_uri_already_saved: bool,
        /// Reset previously saved details if not provided
        #[arg(short = 'r', long = "reset-missing-details")]
        reset_missing: bool,
    },
    /// Save/update multiple bookmarks
    SaveAll {
        /// Uri of the bookmark
        #[arg(value_name = "URI")]
        uris: Option<Vec<String>>,
        /// Tags to attach to the bookmarks
        #[arg(
            short = 't',
            long = "tags",
            value_name = "STRING,STRING..",
            value_delimiter = ','
        )]
        tags: Option<Vec<String>>,
        /// Read input from stdin
        #[arg(short = 's', long = "stdin")]
        use_stdin: bool,
        /// Reset previously saved tags if not provided
        #[arg(short = 'r', long = "reset-missing-details")]
        reset_missing: bool,
    },
    /// Search bookmarks based on a singular query
    Search {
        /// Pattern to match any attribute of a bookmark (URI, title, tags)
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
        /// whether to show results in bmm's TUI
        #[arg(long = "tui")]
        tui: bool,
    },
    /// Show bookmark details
    Show {
        /// URI of the bookmark
        #[arg(value_name = "URI")]
        uri: String,
    },
    /// Interact with tags
    Tags {
        #[command(subcommand)]
        tags_command: TagsCommand,
    },
    /// Open bmm's TUI
    Tui,
}

#[derive(Subcommand, Debug)]
pub enum TagsCommand {
    /// Delete tags
    Delete {
        /// Tags to delete
        #[arg(value_name = "STRING")]
        tags: Vec<String>,
        /// Whether to skip confirmation
        #[arg(short = 'y', long = "yes")]
        skip_confirmation: bool,
    },
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
        /// whether to show results in bmm's TUI
        #[arg(long = "tui")]
        tui: bool,
    },
    /// Rename a tag
    Rename {
        /// Source tag (must already exist)
        #[arg(value_name = "SOURCE")]
        source_tag: String,
        /// Target tag (can either be a new tag or an already existing one)
        #[arg(value_name = "TARGET")]
        target_tag: String,
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
            BmmCommand::Delete {
                uris,
                skip_confirmation,
            } => format!(
                r#"
command           : Delete bookmark(s)
URIs              : {}
skip confirmation : {}
"#,
                uris.join(", "),
                skip_confirmation,
            ),
            BmmCommand::List {
                uri,
                title,
                tags,
                format,
                limit,
            } => format!(
                r#"
command           : List bookmark(s)
URI query         : {}
title query       : {}
tags              : {:?}
format            : {}
limit             : {}
"#,
                uri.as_deref().unwrap_or(NOT_PROVIDED),
                title.as_deref().unwrap_or(NOT_PROVIDED),
                tags,
                format,
                limit,
            ),
            BmmCommand::Import {
                file,
                dry_run,
                reset_missing,
            } => format!(
                r#"
command       : Import bookmarks
file          : {}
dry run       : {}
reset missing : {}
"#,
                file, dry_run, reset_missing
            ),
            BmmCommand::Save {
                uri,
                title,
                tags,
                use_editor,
                fail_if_uri_already_saved,
                reset_missing,
            } => format!(
                r#"
command                   : Save/update bookmark
URI                       : {}
title                     : {}
tags                      : {}
use editor                : {}
fail if URI already saved : {}
reset missing             : {}
"#,
                uri,
                title.as_deref().unwrap_or(NOT_PROVIDED),
                tags.as_ref().map_or(NOT_PROVIDED.into(), |t| t.join(" ")),
                use_editor,
                fail_if_uri_already_saved,
                reset_missing,
            ),
            BmmCommand::SaveAll {
                uris,
                tags,
                use_stdin,
                reset_missing,
            } => format!(
                r#"
command                   : Save/update bookmarks
URIs                      : {}
tags                      : {}
use stdin                 : {}
reset missing             : {}
"#,
                uris.as_ref().map_or(NOT_PROVIDED.into(), |u| u.join(" ")),
                tags.as_ref().map_or(NOT_PROVIDED.into(), |t| t.join(" ")),
                use_stdin,
                reset_missing,
            ),
            BmmCommand::Search {
                query: uri,
                format,
                limit,
                tui,
            } => format!(
                r#"
command     : Search bookmarks
query       : {}
format      : {}
limit       : {}
tui         : {}
"#,
                uri, format, limit, tui
            ),
            BmmCommand::Show { uri } => format!(
                r#"
command     : Show bookmarks
URI         : {}
"#,
                uri
            ),
            BmmCommand::Tags { tags_command } => match tags_command {
                TagsCommand::List {
                    format,
                    show_stats,
                    tui,
                } => format!(
                    r#"
command      : List Tags
format       : {}
show stats   : {}
run tui      : {}
"#,
                    format, show_stats, tui,
                ),
                TagsCommand::Rename {
                    source_tag,
                    target_tag,
                } => format!(
                    r#"
command      : Rename Tag
source tag   : {}
target tag   : {}
"#,
                    source_tag, target_tag,
                ),
                TagsCommand::Delete {
                    tags,
                    skip_confirmation,
                } => format!(
                    r#"
command          : Delete Tags
tags             : {:?}
skip confirmation: {}
"#,
                    tags, skip_confirmation,
                ),
            },
            BmmCommand::Tui => r#"
command      : Open TUI
"#
            .to_string(),
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
