#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

required_binaries=("hyperfine")

check_binaries() {
    for binary in "${required_binaries[@]}"; do
        if ! command -v "$binary" &>/dev/null; then
            echo "Error: $binary is not installed." >&2
            exit 1
        fi
    done
}

check_binaries

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <path-to-prev-binary> <path-to-new-binary>"
    exit 1
fi

prev_bin=$1
new_bin=$2

temp_dir=$(mktemp -d)
echo "temp_dir: $temp_dir"

if [[ ! -d "$temp_dir" ]]; then
    echo "Error: Failed to create temporary directory." >&2
    exit 1
fi

tags=$(printf "tag%s," {1..100} | sed 's/,$/\n/')
echo "100 tags"

data_file="${temp_dir}/bookmarks.html"

./generate-data.py 8000 "${tags}" "${data_file}"

${prev_bin} --db-path "${temp_dir}/bmm.db" import "${data_file}" >/dev/null

num_bookmarks=$(bmm --db-path "${temp_dir}/bmm.db" list -f plain -l 8000 | wc -l | xargs)

cat <<EOF

---
There are "${num_bookmarks}" bookmarks.
---
EOF

cat <<EOF

# --------------------------- #
# 1. Searching for a keyword  #
# --------------------------- #

EOF

PREV_COMMAND="${prev_bin} --db-path ${temp_dir}/bmm.db search -f plain -l 500 1000.com"
NEW_COMMAND="${new_bin} --db-path ${temp_dir}/bmm.db search -f plain -l 500 1000.com"

hyperfine --warmup 50 --runs 300 "${PREV_COMMAND}" "${NEW_COMMAND}" -n prev -n new

cat <<EOF

# ---------------------------------- #
# 2. Searching for several keywords  #
# ---------------------------------- #

EOF

PREV_COMMAND="${prev_bin} --db-path ${temp_dir}/bmm.db search -f plain -l 500 'tag1 tag2 tag3 tag4 tag5'"
NEW_COMMAND="${new_bin} --db-path ${temp_dir}/bmm.db search -f plain -l 500 'tag1 tag2 tag3 tag4 tag5'"

hyperfine --warmup 50 --runs 300 "${PREV_COMMAND}" "${NEW_COMMAND}" -n prev -n new

cat <<EOF

# --------------------- #
# 2. Listing by a tag   #
# --------------------- #

EOF

PREV_COMMAND="${prev_bin} --db-path ${temp_dir}/bmm.db list -f plain -l 1000 -t tag1"
NEW_COMMAND="${new_bin} --db-path ${temp_dir}/bmm.db list -f plain -l 1000 -t tag1"

hyperfine --warmup 50 --runs 300 "${PREV_COMMAND}" "${NEW_COMMAND}" -n prev -n new

trap 'rm -rf "$temp_dir"' EXIT
