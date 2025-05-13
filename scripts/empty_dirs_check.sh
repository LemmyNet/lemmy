#!/usr/bin/env bash
set -e

EMPTY_DIRS=$(find crates migrations api_tests/src -type d -empty)
if [[ "$EMPTY_DIRS" ]]; then
  logger -s "Empty dirs present:\n$EMPTY_DIRS\n"
fi
