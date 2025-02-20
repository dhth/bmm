<p align="center">
  <h1 align="center">bmm</h1>
  <p align="center">
    <a href="https://github.com/dhth/bmm/actions/workflows/build.yml"><img alt="GitHub release" src="https://img.shields.io/github/actions/workflow/status/dhth/bmm/build.yml?style=flat-square"></a>
    <a href="https://crates.io/crates/bmm"><img alt="GitHub release" src="https://img.shields.io/crates/v/bmm?style=flat-square"></a>
    <a href="https://github.com/dhth/bmm/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/release/dhth/bmm.svg?style=flat-square"></a>
    <a href="https://github.com/dhth/bmm/releases"><img alt="Commits since latest release" src="https://img.shields.io/github/commits-since/dhth/bmm/latest?style=flat-square"></a>
  </p>
</p>

`bmm` (stands for "bookmarks manager") lets you get to your bookmarks in a
flash.

![tui-2](https://github.com/user-attachments/assets/a3dc5fb7-d258-461e-86b5-f2498dfbd4dc)

It does so by storing your bookmarks locally, allowing you to quickly access,
manage, and search through them using various commands. `bmm` has a traditional
command line interface that can be used standalone and/or integrated with other
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

**homebrew**:

```sh
brew install dhth/tap/bmm
```

**cargo**:

```sh
cargo install bmm
```

Or get the binaries directly from a Github [release][1]. Read more about
verifying the authenticity of released artifacts
[here](#-verifying-release-artifacts).

⚡️ Usage
---

```text
Usage: bmm [OPTIONS] <COMMAND>

Commands:
  import    Import bookmarks from various sources
  delete    Delete bookmarks
  list      List bookmarks based on several kinds of queries
  save      Save/update a bookmark
  save-all  Save/update multiple bookmarks
  search    Search bookmarks based on a singular query
  show      Show bookmark details
  tags      Interact with tags
  tui       Open bmm's TUI
  help      Print this message or the help of the given subcommand(s)

Options:
      --db-path <STRING>  Override bmm's database location (default: <DATA_DIR>/bmm/bmm.db)
      --debug             Output debug information without doing anything
  -h, --help              Print help (see more with '--help')
```

⌨ CLI mode
---

`bmm` allows every action it supports to be performed via its CLI. As such, it
can be easily integrated with other search tools (eg.
[Alfred](https://www.alfredapp.com/), [fzf](https://github.com/junegunn/fzf),
etc.)

![cli](https://github.com/user-attachments/assets/f8493e7c-8286-4fa4-8d49-6f34b5c5044b)

### Importing existing bookmarks

`bmm` allows Import bookmarks from various sources. It supports the following
input formats:

- HTML (These are bookmark files exported by browsers like Firefox, Chrome, etc,
  in the NETSCAPE-Bookmark-file-1 format.)
- JSON
- TXT

```bash
bmm import firefox.html
bmm import bookmarks.json --dry-run
bmm import bookmarks.txt --reset-missing-details
```

<details><summary> An example HTML file</summary>

```html
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<!-- This is an automatically generated file.
     It will be read and overwritten.
     DO NOT EDIT! -->
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self'; script-src 'none'; img-src data: *; object-src 'none'"></meta>
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks Menu</H1>

<DL><p>
    <DT><H3 ADD_DATE="1736450822" LAST_MODIFIED="1739920697" PERSONAL_TOOLBAR_FOLDER="true">Bookmarks Toolbar</H3>
    <DL><p>
        <DT><H3 ADD_DATE="1739896938" LAST_MODIFIED="1739920670">productivity</H3>
        <DL><p>
            <DT><H3 ADD_DATE="1739896992" LAST_MODIFIED="1739920767">crates</H3>
            <DL><p>
                <DT><A HREF="https://crates.io/crates/sqlx" ADD_DATE="1739897020" LAST_MODIFIED="1739897041" ICON_URI="https://crates.io/favicon.ico" TAGS="crates,rust">sqlx - crates.io: Rust Package Registry</A>
            </DL><p>
            <DT><A HREF="https://github.com/dhth/omm" ADD_DATE="1739920615" LAST_MODIFIED="1739920646" ICON_URI="https://github.com/fluidicon.png" TAGS="productivity,tools">GitHub - dhth/omm: on-my-mind: a keyboard-driven task manager for the command line</A>
            <DT><A HREF="https://github.com/dhth/hours" ADD_DATE="1739920661" LAST_MODIFIED="1739920670" ICON_URI="https://github.com/fluidicon.png" TAGS="productivity,tools">GitHub - dhth/hours: A no-frills time tracking toolkit for command line nerds</A>
        </DL><p>
        <DT><A HREF="https://github.com/dhth/bmm" ADD_DATE="1739920697" LAST_MODIFIED="1739920739" ICON_URI="https://github.com/fluidicon.png" TAGS="tools">GitHub - dhth/bmm: get to your bookmarks in a flash</A>
    </DL><p>
</DL>
```
</details>

<details><summary> An example JSON file</summary>

```json
[
  {
    "uri": "https://github.com/dhth/bmm",
    "title": null,
    "tags": "tools,bookmarks"
  },
  {
    "uri": "https://github.com/dhth/omm",
    "title": "on-my-mind: a keyboard-driven task manager for the command line",
    "tags": null
  }
]
```
</details>

<details><summary> An example TXT file</summary>

```text
https://github.com/dhth/bmm
https://github.com/dhth/omm
https://github.com/dhth/hours
```
</details>

### Saving/updating a bookmark

```bash
# save a new URI
bmm save https://github.com/dhth/bmm

# save a new URI with title and tags
bmm save https://github.com/dhth/omm \
    --title 'a keyboard-driven task manager for the command line' \
    --tags 'tools,productivity'

# update the title of a previously saved bookmark
bmm save https://github.com/dhth/bmm \
    --title 'yet another bookmarking tool'

# append to the tags of a previously saved bookmark
bmm save https://github.com/dhth/omm \
    --tags 'task-manager'

# use your editor to provide details
bmm save https://github.com/dhth/bmm -e
```

### Saving/updating several bookmarks at a time

```bash
# save/update multiple bookmarks via arguments
bmm save \
    'https://github.com/dhth/bmm' \
    'https://github.com/dhth/omm' \
    --tags 'cli,bookmarks'

# save/update multiple bookmarks via stdin
cat << EOF | bmm save --tags tools --reset-missing-details -s
https://github.com/dhth/bmm
https://github.com/dhth/omm
https://github.com/dhth/hours
EOF
```

### Listing bookmarks based on several queries

`bmm` allows listing bookmarks based on queries on bookmark uri/title/tags. The
first two are pattern matched, while the last is matched exactly.

```bash
bmm list --uri 'github.com' \
    --title 'command line' \
    --tags 'tools,productivity' \
    --format json
```

### Searching bookmarks based on a singular query

Sometimes you want to search for bookmarks without being very granular. The
`search` command pattern matches a search query over all bookmarks attributes
and tags and returns the results. You can also open the results in `bmm`'s TUI.

```bash
# search bookmarks based on a singular query
bmm search 'cli' --format delimited

# open search results in bmm's TUI
bmm search 'cli' --tui
```

### Show bookmark details

```bash
bmm show 'https://github.com/dhth/bmm'
```

### Interaction with tags

```bash
# Show saved tags
bmm tags list \
    --format json \
    --show-stats

# open saved tags in bmm's TUI
bmm tags list --tui

# rename tag
bmm tags rename old-tag new-tag

# delete tags 
bmm tags delete tag1 tag2 tag3
```

### Delete bookmarks

```bash
bmm delete 'https://github.com/dhth/bmm' 'https://github.com/dhth/omm'

# skip confirmation
bmm delete --yes 'https://github.com/dhth/bmm'
```

📟 TUI mode
---

To allow for easy browsing, `bmm` ships with its own TUI. It can be launched
either in a generic mode (via `bmm tui`) or in the context of a specific command
(e.g., `bmm search tools --tui`).

The TUI lets you do the following:

- Search bookmarks based on a query
- List all tags
- View bookmarks that hold a tag

Feature requests for the TUI can be submitted via `bmm`'s [issues
page](https://github.com/dhth/bmm/issues).

![tui](https://github.com/user-attachments/assets/6ca63039-8872-4520-93da-1576cc0cf8ec)

🔐 Verifying release artifacts
---

In case you get the `bmm` binary directly from a [release][1], you may want to
verify its authenticity. Checksums are applied to all released artifacts, and
the resulting checksum file is attested using [Github Attestations][2].

Steps to verify (replace `A.B.C` in the commands below with the version you
want):

1. Download the sha256 checksum file for your platform from the release:

   ```shell
   curl -sSLO https://github.com/dhth/bmm/releases/download/vA.B.C/bmm-x86_64-unknown-linux-gnu.tar.xz.sha256
   ```

2. Verify the integrity of the checksum file using [gh][3].

   ```shell
   gh attestation verify bmm-x86_64-unknown-linux-gnu.tar.xz.sha256 --repo dhth/bmm
   ```

3. Download the compressed archive you want, and validate its checksum:

   ```shell
   curl -sSLO https://github.com/dhth/bmm/releases/download/vA.B.C/bmm-x86_64-unknown-linux-gnu.tar.xz
   sha256sum --ignore-missing -c bmm-x86_64-unknown-linux-gnu.tar.xz.sha256
   ```

3. If checksum validation goes through, uncompress the archive:

   ```shell
   tar -xzf bmm-x86_64-unknown-linux-gnu.tar.xz
   cd bmm-x86_64-unknown-linux-gnu
   ./bmm
   # profit!
   ```

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

[1]: https://github.com/dhth/bmm/releases
[2]: https://github.blog/news-insights/product-news/introducing-artifact-attestations-now-in-public-beta/
[3]: https://github.com/cli/cli
