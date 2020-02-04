# Goals
- Come up with a name / codename.
- Must have communities.
- Must have threaded comments.
- Must be federated: liking and following communities across instances.
- Be live-updating: have a right pane for new comments, and a main pain for the full threaded view.
  - Use websockets for post / gets to your own instance.

# Questions
- How does voting work? Should we go back to the old way of showing up and downvote counts? Or just a score?
- Decide on tech to be used
  - Backend: Actix, Diesel.
  - Frontend: inferno, typescript and bootstrap for now.
- Should it allow bots?
- Should the comments / votes be static, or feel like a chat, like [flowchat?](https://flow-chat.com).
  - Two pane model - Right pane is live comments, left pane is live tree view.
  - On mobile, allow you to switch between them. Default?

# Resources / Potential Libraries
- Use the [activitypub crate.](https://docs.rs/activitypub/0.1.4/activitypub/)
- https://docs.rs/activitypub/0.1.4/activitypub/
- [Activitypub vocab.](https://www.w3.org/TR/activitystreams-vocabulary/)
- [Activitypub main](https://www.w3.org/TR/activitypub/)
- [Diesel to Postgres data types](https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html)
- [helpful diesel examples](http://siciarz.net/24-days-rust-diesel/)
- [Recursive query for adjacency list for nested comments](https://stackoverflow.com/questions/192220/what-is-the-most-efficient-elegant-way-to-parse-a-flat-table-into-a-tree/192462#192462)
- https://github.com/sparksuite/simplemde-markdown-editor
- [Markdown-it](https://github.com/markdown-it/markdown-it)
- [Sticky Sidebar](https://stackoverflow.com/questions/38382043/how-to-use-css-position-sticky-to-keep-a-sidebar-visible-with-bootstrap-4/49111934)
- [RXJS websocket](https://stackoverflow.com/questions/44060315/reconnecting-a-websocket-in-angular-and-rxjs/44067972#44067972)
- [Rust JWT](https://github.com/Keats/jsonwebtoken)
- [Hierarchical tree building javascript](https://stackoverflow.com/a/40732240/1655478)
- [Hot sorting discussion](https://meta.stackexchange.com/questions/11602/what-formula-should-be-used-to-determine-hot-questions) [2](https://medium.com/hacking-and-gonzo/how-reddit-ranking-algorithms-work-ef111e33d0d9)
- [Classification types.](https://www.reddit.com/r/ModeratorDuck/wiki/subreddit_classification)
- [RES expando - Possibly make this into a switching react component.](https://github.com/honestbleeps/Reddit-Enhancement-Suite/tree/d21f55c21e734f47d8ed03fe0ebce5b16653b0bd/lib/modules/hosts)
- [Temp Icon](https://www.flaticon.com/free-icon/mouse_194242)
- [Rust docker build](https://shaneutt.com/blog/rust-fast-small-docker-image-builds/)
- [Zurb mentions](https://github.com/zurb/tribute)
- Activitypub guides
  - https://blog.joinmastodon.org/2018/06/how-to-implement-a-basic-activitypub-server/
  - https://raw.githubusercontent.com/w3c/activitypub/gh-pages/activitypub-tutorial.txt
  - https://github.com/tOkeshu/activitypub-example
  - https://blog.joinmastodon.org/2018/07/how-to-make-friends-and-verify-requests/

