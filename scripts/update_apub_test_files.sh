#!/bin/bash
set -e

curl -H "Accept: application/activity+json" https://lemmy.ml/u/nutomic | jq \
    > crates/apub/assets/lemmy-person.json
curl -H "Accept: application/activity+json" https://lemmy.ml/c/announcements | jq \
    > crates/apub/assets/lemmy-community.json
# replace these collection links so that tests dont make any actual http requests
sed -i 's/https:\/\/lemmy.ml\/c\/announcements\/outbox/https:\\/\\/lemmy.ml\\/c\\/announcements\\/not_outbox/g' crates/apub/assets/lemmy-community.json
sed -i 's/https:\/\/lemmy.ml\/c\/announcements\/moderators/https:\\/\\/lemmy.ml\\/c\\/announcements\\/not_moderators/g' crates/apub/assets/lemmy-community.json
curl -H "Accept: application/activity+json" https://lemmy.ml/post/55143 | jq \
    > crates/apub/assets/lemmy-post.json
curl -H "Accept: application/activity+json" https://queer.hacktivis.me/users/lanodan | jq \
    > crates/apub/assets/pleroma-person.json
