#!/bin/bash
set -e

cargo workspaces exec cargo clippy --fix --allow-dirty --tests --all-targets --all-features -- \
    -D warnings -D deprecated -D clippy::perf -D clippy::complexity \
    -D clippy::dbg_macro -D clippy::inefficient_to_string \
    -D clippy::items-after-statements -D clippy::implicit_clone \
    -D clippy::wildcard_imports -D clippy::cast_lossless \
    -D clippy::manual_string_new -D clippy::redundant_closure_for_method_calls
