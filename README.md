<p align="center">
  <h1 align="center">bmm</h1>
  <p align="center">
    <a href="https://github.com/dhth/bmm/actions/workflows/build.yml"><img alt="GitHub release" src="https://img.shields.io/github/actions/workflow/status/dhth/bmm/build.yml?style=flat-square"></a>
  </p>
</p>

`bmm` (stands for "bookmarks manager") lets you get to your bookmarks in a
flash.

![tui-2](https://github.com/user-attachments/assets/a3dc5fb7-d258-461e-86b5-f2498dfbd4dc)

It does so by storing your bookmarks locally, allowing you to quickly access,
manage, and search through them using various commands. `bmm` has a traditional
command line interface that can be used standalone or integrated with other
tools, and a textual user interface for easy browsing.

🤔 Motivation
---

I'd been using [buku](https://github.com/jarun/buku) for managing my bookmarks
via the command line. It's a fantastic tool, but I was noticing some slowdown
after years of collecting bookmarks in it. I was curious if I could replicate
the subset of its functionality that I used while improving search performance.
Additionally, I missed having a TUI to browse bookmarks in. `bmm` started out as
a way to fulfill both goals. Turns out, it runs quite a lot faster than `buku`
(check out benchmarks
[here](https://github.com/dhth/bmm/actions/workflows/bench.yml)). I've now moved
my bookmark management completely to `bmm`, but `buku` remains an excellent
tool, and those looking for a broader feature set should definitely check it
out.

💾 Installation
---

**cargo**:

```sh
cargo install --git https://github.com/dhth/bmm.git
```

⚡️ Usage
---

```text
Usage: bmm [OPTIONS] <COMMAND>

Commands:
  import  Import bookmarks from various sources
  delete  Delete bookmarks
  list    List bookmarks based on several kinds of queries
  save    Save/update a bookmark
  search  Search bookmarks based on a singular query
  show    Show bookmark details
  tags    Interact with bmm tags
  tui     Open bmm's TUI
  help    Print this message or the help of the given subcommand(s)

Options:
      --db-path <STRING>  override bmm's database location (default: <DATA_DIR>/bmm/bmm.db)
      --debug             output debug information without doing anything
  -h, --help              Print help
```

### Basic Usage

```bash

# import bookmarks
bmm import firefox.html
bmm import bookmarks.json --dry-run
bmm import bookmarks.txt --reset-missing-details

# save a new URI
bmm save https://github.com/dhth/bmm

# update the title of a previously saved bookmark
bmm save https://github.com/dhth/bmm --title "yet another bookmarking tool"

# update the tags of a previously saved bookmark
bmm save https://github.com/dhth/bmm \
    --tags cli,bookmarks

# use your editor to provide details
bmm save https://github.com/dhth/bmm -e

# list bookmarks based on several queries
bmm list --uri 'github.com' \
    --title 'cli tool' \
    --tags cli \
    --format json

# search bookmarks based on a singular query
bmm search "cli" --format delimited

# open search results in bmm's TUI
bmm search "cli" --tui

# show details for a bookmark
bmm show https://github.com/dhth/bmm

# show saved tags
bmm tags list \
    --format json \
    --show-stats

# open saved tags in bmm's TUI
bmm tags list --tui

# delete tags 
bmm tags delete tag1 tag2 tag3

# delete bookmarks and skip confirmation
bmm delete --yes https://github.com/dhth/bmm https://github.com/dhth/omm

# rename tag
bmm tags rename old-tag new-tag

# open bmm's TUI
bmm tui
```

⌨ CLI mode
---

`bmm` allows every action it supports to be performed via its CLI. As such, it
can be easily integrated with other search tools (eg.
[Alfred](https://www.alfredapp.com/), [fzf](https://github.com/junegunn/fzf),
etc.)

![cli](https://github.com/user-attachments/assets/f8493e7c-8286-4fa4-8d49-6f34b5c5044b)

📟 TUI mode
---

To allow for quick access, `bmm` ships with its own TUI. The TUI simplifies
browsing with a user-friendly interface. It can be launched either in a generic
mode (via `bmm tui`) or in the context of a specific command (e.g., `bmm search
tools --tui`).

![tui](https://github.com/user-attachments/assets/6ca63039-8872-4520-93da-1576cc0cf8ec)

🙏 Acknowledgements
---

`bmm` sits on the shoulders of the following crates:

- [clap](https://crates.io/crates/clap)
- [csv](https://crates.io/crates/csv)
- [dirs](https://crates.io/crates/dirs)
- [lazy_static](https://crates.io/crates/lazy_static)
- [once_cell](https://crates.io/crates/once_cell)
- [open](https://crates.io/crates/open)
- [ratatui](https://crates.io/crates/ratatui)
- [regex](https://crates.io/crates/regex)
- [select](https://crates.io/crates/select)
- [serde](https://crates.io/crates/serde)
- [serde_json](https://crates.io/crates/serde_json)
- [sqlx](https://crates.io/crates/sqlx)
- [tempfile](https://crates.io/crates/tempfile)
- [thiserror](https://crates.io/crates/thiserror)
- [tokio](https://crates.io/crates/tokio)
- [input](https://crates.io/crates/tui-input)
- [url](https://crates.io/crates/url)
- [which](https://crates.io/crates/which)
