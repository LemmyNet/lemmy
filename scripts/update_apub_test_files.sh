#!/bin/bash
set -e

curl -H "Accept: application/activity+json" https://lemmy.ml/u/nutomic | jq \
    > crates/apub/assets/lemmy-person.json
curl -H "Accept: application/activity+json" https://lemmy.ml/c/meta | jq \
    > crates/apub/assets/lemmy-community.json
curl -H "Accept: application/activity+json" https://lemmy.ml/post/70280 | jq \
    > crates/apub/assets/lemmy-post.json
curl -H "Accept: application/activity+json" https://queer.hacktivis.me/users/lanodan | jq \
    > crates/apub/assets/pleroma-person.json
