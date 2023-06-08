#!/usr/bin/env bash
set -e

times=3
duration=0
for ((i=0; i < times; i++)) ; do
    echo "Starting iteration $i"
    echo "cargo clean"
    # to benchmark incremental compilation time, do a full build with the same compiler version first,
    # and use the following clean command:
    cargo clean -p lemmy_utils
    #cargo clean
    echo "cargo build"
    start=$(date +%s.%N)
    RUSTC_WRAPPER='' cargo build -q
    end=$(date +%s.%N)
    echo "Finished iteration $i after $(bc <<< "scale=0; $end - $start") seconds"
    duration=$(bc <<< "$duration + $end - $start")
done

average=$(bc <<< "scale=0; $duration / $times")

echo "Average compilation time over $times runs is $average seconds"
