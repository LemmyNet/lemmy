#!/bin/sh

declare -a arr=(
"https://mastodon.social/"
"https://peertube.social/"
"https://dev.lemmy.ml/"
"https://dev.lemmy.ml/feeds/all.xml"
"https://dev.lemmy.ml/.well-known/nodeinfo"
"https://fediverse.blog/.well-known/nodeinfo"
"https://torrents-csv.ml/service/search?q=wheel&page=1&type_=torrent"
)

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
