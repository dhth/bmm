#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

required_binaries=("buku" "bmm" "hyperfine")

check_binaries() {
    for binary in "${required_binaries[@]}"; do
        if ! command -v "$binary" &>/dev/null; then
            echo "Error: $binary is not installed." >&2
            exit 1
        fi
    done
}

check_binaries

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
XDG_DATA_HOME="${temp_dir}" buku --nostdin --offline --import "${data_file}" >/dev/null <<EOF
n
n
n
EOF

bmm --db-path="${temp_dir}/bmm.db" import "${data_file}" >/dev/null

buku_num=$(XDG_DATA_HOME="${temp_dir}" buku --nostdin --np --nc -p -f 10 | wc -l | xargs)
bmm_num=$(bmm --db-path="${temp_dir}/bmm.db" list -f plain -l 8000 | wc -l | xargs)

cat <<EOF

---
buku has "${buku_num}" bookmarks.
bmm  has "${bmm_num}" bookmarks.
---
EOF

cat <<EOF

# --------------------------- #
# 1. Searching for a keyword  #
# --------------------------- #
EOF

XDG_DATA_HOME=${temp_dir} buku --nostdin --np -f 10 --nc -n 500 -s 1000.com >/var/tmp/buku.txt
bmm --db-path "${temp_dir}/bmm.db" search -f plain -l 500 1000.com >/var/tmp/bmm.txt

git --no-pager diff --no-index /var/tmp/buku.txt /var/tmp/bmm.txt || {
    echo "command outputs differ"
    exit 1
}

BUKU_COMMAND="XDG_DATA_HOME=${temp_dir} buku --nostdin --np -f 10 --nc -n 500 -s 1000.com"
BMM_COMMAND="bmm --db-path=${temp_dir}/bmm.db search -f plain -l 500 1000.com"

cat <<EOF

$BUKU_COMMAND
 v.
$BMM_COMMAND

EOF

hyperfine --warmup 30 --runs 100 "${BUKU_COMMAND}" "${BMM_COMMAND}" -n buku -n bmm

cat <<EOF

# --------------------- #
# 2. Listing by a tag   #
# --------------------- #
EOF

XDG_DATA_HOME=${temp_dir} buku --nostdin --np -f 10 --nc -n 1000 --stag tag1 > buku.txt
bmm --db-path "${temp_dir}/bmm.db" list -f plain -l 1000 -t tag1 >bmm.txt

git --no-pager diff --no-index /var/tmp/buku.txt /var/tmp/bmm.txt || {
    echo "command outputs differ"
    exit 1
}

BUKU_COMMAND="XDG_DATA_HOME=${temp_dir} buku --nostdin --np -f 10 --nc -n 1000 --stag tag1"
BMM_COMMAND="bmm --db-path=${temp_dir}/bmm.db list -f plain -l 1000 -t tag1"

cat <<EOF

$BUKU_COMMAND
 v.
$BMM_COMMAND

EOF

hyperfine --warmup 30 --runs 100 "${BUKU_COMMAND}" "${BMM_COMMAND}" -n buku -n bmm

trap 'rm -rf "$temp_dir"' EXIT
