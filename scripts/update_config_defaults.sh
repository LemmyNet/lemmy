#!/bin/bash
set -e

dest=${1-config/defaults.hjson}

cargo run -- --print-config-docs > "$dest"
# replace // comments with #
sed -i "s/^\([[:space:]]*\)\/\//\1#/" "$dest"
# remove trailing commas
sed -i "s/,\$//" "$dest"
# remove quotes around json keys
sed -i "s/\"//" "$dest"
sed -i "s/\"//" "$dest"
