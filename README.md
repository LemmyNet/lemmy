# Rust Reddit Fediverse (to be renamed later)

We have a twitter alternative (mastodon), a facebook alternative (friendica), so let's build a reddit alternative in the fediverse.

[Matrix Chatroom](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)

## TODOs

- Use the [activitypub crate.](https://docs.rs/activitypub/0.1.4/activitypub/)
- https://docs.rs/activitypub/0.1.4/activitypub/
- [Activitypub vocab.](https://www.w3.org/TR/activitystreams-vocabulary/)
- Create a markdown doc of actions, matching up to things in that vocab.

## Goals

- Come up with a name / codename.
- Must have communities.
- Must have threaded comments.
- Must be federated: liking and following communities across instances.

## Questions

- How does voting work? Should we go back to the old way of showing up and downvote counts? Or just a score?
- Decide on tech to be used
  - Backend: Actix, Diesel.
  - Frontend: inferno, typescript and bootstrap for now.
- Should it allow bots?
- Should the comments / votes be static, or feel like a chat, like [flowchat?](https://flow-chat.com). 
