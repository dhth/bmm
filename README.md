# bmm

`bmm` (stands for "bookmarks manager") lets you get to your bookmarks in a
flash.

> [!NOTE]
> `bmm` is alpha software, and is under active development. Its interface might
> change till it's stable.

Usage
---

```text
Usage: bmm [OPTIONS] <COMMAND>

Commands:
  import  Import bookmarks from various sources
  list    List bookmarks based on several kinds of queries
  save    Saves a bookmark
  search  Search bookmarks based on a singular query
  show    Show bookmark details
  tags    Interact with bmm tags
```

### Basic Usage

```bash
# save just the uri
bmm save https://github.com/dhth/bmm

# save a title as well
bmm save https://github.com/dhth/bmm --title "yet another bookmarking tool"

# save tags as well
bmm save https://github.com/dhth/bmm \
    --title "yet another bookmarking tool" \
    --tags cli,bookmarks

# use your editor to provide details
bmm save https://github.com/dhth/bmm -e

# save or update bookmark
bmm save https://github.com/dhth/bmm --title "updated title"

# list bookmarks based on several queries
bmm list --uri 'github.com' \
    --title 'cli tool' \
    --tags cli \
    --format json

# search bookmarks based on a singular query
bmm list "cli" \
    --format delimited

# show details for a bookmark
bmm show https://github.com/dhth/bmm

# show saved tags
bmm tags list \
    --format json \
    --show-stats

# delete bookmarks
bmm delete --yes https://github.com/dhth/bmm https://github.com/dhth/omm

# rename tag
bmm tags rename old-tag new-tag
```
