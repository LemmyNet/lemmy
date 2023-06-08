#!/usr/bin/env bash
set -e

declare -a arr=(
"https://mastodon.social/"
"https://peertube.social/"
"https://lemmy.ml/"
"https://lemmy.ml/feeds/all.xml"
"https://lemmy.ml/.well-known/nodeinfo"
"https://fediverse.blog/.well-known/nodeinfo"
"https://torrents-csv.ml/service/search?q=wheel&page=1&type_=torrent"
)

## check if ab installed
if ! [ -x "$(command -v ab)" ]; then
  echo 'Error: ab (Apache Bench) is not installed. https://httpd.apache.org/docs/2.4/programs/ab.html' >&2
  exit 1
fi

## now loop through the above array
for i in "${arr[@]}"
do
  ab -c 10 -t 10 "$i" > out.abtest
  grep "Server Hostname:" out.abtest
  grep "Document Path:" out.abtest
  grep "Requests per second" out.abtest
  grep "(mean, across all concurrent requests)" out.abtest
  grep "Transfer rate:" out.abtest
  echo "---"
done

rm *.abtest
