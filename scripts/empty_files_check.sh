#!/usr/bin/env bash
set -e

# Makes sure there are no files smaller than 2 bytes
# Don't use completely empty, as some editors use newlines
EMPTY_FILES=$(find crates migrations api_tests/src config -type f -size -2c)
if [[ "$EMPTY_FILES" ]]; then echo "Empty files present:\n$EMPTY_FILES\n" && exit 1; fi
