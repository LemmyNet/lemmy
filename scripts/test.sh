#!/bin/sh

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

set -e

psql -U lemmy -d postgres -c "DROP DATABASE lemmy;"
psql -U lemmy -d postgres -c "CREATE DATABASE lemmy;"

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
# tests are executed in working directory crates/api (or similar),
# so to load the config we need to traverse to the repo root
export LEMMY_CONFIG_LOCATION=../../config/config.hjson
RUST_BACKTRACE=1 \
  cargo test --workspace --no-fail-fast
# Add this to do printlns: -- --nocapture
