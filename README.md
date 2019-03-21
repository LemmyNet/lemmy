# Rust Reddit Fediverse (to be renamed later)

We have a twitter alternative (mastodon), a facebook alternative (friendica), so let's build a reddit alternative in the fediverse.

[Matrix Chat: #rust-reddit-fediverse:matrix.org](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)

[ActivityPub API.md](API.md)

## Goals
- Come up with a name / codename.
- Must have communities.
- Must have threaded comments.
- Must be federated: liking and following communities across instances.
- Be live-updating: have a right pane for new comments, and a main pain for the full threaded view.
  - Use websockets for post / gets to your own instance.

## Questions
- How does voting work? Should we go back to the old way of showing up and downvote counts? Or just a score?
- Decide on tech to be used
  - Backend: Actix, Diesel.
  - Frontend: inferno, typescript and bootstrap for now.
- Should it allow bots?
- Should the comments / votes be static, or feel like a chat, like [flowchat?](https://flow-chat.com).
  - Two pane model - Right pane is live comments, left pane is live tree view.
  - On mobile, allow you to switch between them. Default?

## Resources / Potential Libraries
- Use the [activitypub crate.](https://docs.rs/activitypub/0.1.4/activitypub/)
- https://docs.rs/activitypub/0.1.4/activitypub/
- [Activitypub vocab.](https://www.w3.org/TR/activitystreams-vocabulary/)
- [Activitypub main](https://www.w3.org/TR/activitypub/)
- [Diesel to Postgres data types](https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html)
- [helpful diesel examples](http://siciarz.net/24-days-rust-diesel/)
- [Mastodan public key server example](https://blog.joinmastodon.org/2018/06/how-to-implement-a-basic-activitypub-server/)
- [Recursive query for adjacency list for nested comments](https://stackoverflow.com/questions/192220/what-is-the-most-efficient-elegant-way-to-parse-a-flat-table-into-a-tree/192462#192462)
- https://github.com/sparksuite/simplemde-markdown-editor
- [Sticky Sidebar](https://stackoverflow.com/questions/38382043/how-to-use-css-position-sticky-to-keep-a-sidebar-visible-with-bootstrap-4/49111934)

## TODOs 
- Endpoints
- DB
- Followers / following


