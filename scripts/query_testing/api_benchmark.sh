#!/usr/bin/env bash
set -e

# By default, this script runs against `http://127.0.0.1:8536`, but you can pass a different Lemmy instance,
# eg `./api_benchmark.sh "https://example.com"`.
DOMAIN=${1:-"http://127.0.0.1:8536"}

declare -a arr=(
"/api/v1/site"
"/api/v1/categories"
"/api/v1/modlog"
"/api/v1/search?q=test&type_=Posts&sort=Hot"
"/api/v1/community"
"/api/v1/community/list?sort=Hot"
"/api/v1/post/list?sort=Hot&type_=All"
)

## check if ab installed
if ! [ -x "$(command -v ab)" ]; then
  echo 'Error: ab (Apache Bench) is not installed. https://httpd.apache.org/docs/2.4/programs/ab.html' >&2
  exit 1
fi

## now loop through the above array
for path in "${arr[@]}"
do
  URL="$DOMAIN$path"
  printf "\n\n\n"
  echo "testing $URL"
  curl --show-error --fail --silent "$URL" >/dev/null
  ab -c 64 -t 10 "$URL" > out.abtest
  grep "Server Hostname:" out.abtest
  grep "Document Path:" out.abtest
  grep "Requests per second" out.abtest
  grep "(mean, across all concurrent requests)" out.abtest
  grep "Transfer rate:" out.abtest
  echo "---"
done

rm *.abtest
