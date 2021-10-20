#!/bin/bash
set -e

curl -H "Accept: application/activity+json" https://lemmy.ml/u/nutomic | jq -S \
    > crates/apub/assets/lemmy-person.json
curl -H "Accept: application/activity+json" https://lemmy.ml/c/announcements | jq -S \
    > crates/apub/assets/lemmy-community.json
# replace these collection links so that tests dont make any actual http requests
sed -i 's/https:\/\/lemmy.ml\/c\/announcements\/outbox/https:\/\/lemmy.ml\/c\/announcements\/not_outbox/g' crates/apub/assets/lemmy-community.json
sed -i 's/https:\/\/lemmy.ml\/c\/announcements\/moderators/https:\/\/lemmy.ml\/c\/announcements\/not_moderators/g' crates/apub/assets/lemmy-community.json
curl -H "Accept: application/activity+json" https://lemmy.ml/post/55143 | jq -S \
    > crates/apub/assets/lemmy-post.json
curl -H "Accept: application/activity+json" https://lemmy.ml/comment/38741 | jq -S \
    > crates/apub/assets/lemmy-comment.json
# replace attributed_to user, so that it takes the same one from above
sed -i 's/https:\/\/lemmy.ml\/u\/my_test/https:\/\/lemmy.ml\/u\/nutomic/g' crates/apub/assets/lemmy-comment.json

curl -H "Accept: application/activity+json" https://queer.hacktivis.me/users/lanodan | jq -S \
    > crates/apub/assets/pleroma-person.json
curl -H "Accept: application/activity+json" https://queer.hacktivis.me/objects/8d4973f4-53de-49cd-8c27-df160e16a9c2 | jq -S \
    > crates/apub/assets/pleroma-comment.json
# rewrite comment inReplyTo so that it refers to our post above (cause lemmy doesnt support standalone comments)
sed -i 's/https:\/\/pleroma.popolon.org\/objects\/bf84a0fb-2ec2-4dff-a1d9-6b573f94fb16/https:\/\/lemmy.ml\/post\/55143/g' crates/apub/assets/pleroma-comment.json

