# Lemmy

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
- [Markdown-it](https://github.com/markdown-it/markdown-it)
- [Sticky Sidebar](https://stackoverflow.com/questions/38382043/how-to-use-css-position-sticky-to-keep-a-sidebar-visible-with-bootstrap-4/49111934)
- [RXJS websocket](https://stackoverflow.com/questions/44060315/reconnecting-a-websocket-in-angular-and-rxjs/44067972#44067972)
- [Rust JWT](https://github.com/Keats/jsonwebtoken)
- [Hierarchical tree building javascript](https://stackoverflow.com/a/40732240/1655478)
- [Hot sorting discussion](https://meta.stackexchange.com/questions/11602/what-formula-should-be-used-to-determine-hot-questions) [2](https://medium.com/hacking-and-gonzo/how-reddit-ranking-algorithms-work-ef111e33d0d9)
- [Classification types.](https://www.reddit.com/r/ModeratorDuck/wiki/subreddit_classification)

## TODOs 
- Endpoints
- DB
- Followers / following

# Trending / Hot / Best Sorting algorithm
## Goals
- During the day, new posts and comments should be near the top, so they can be voted on.
- After a day or so, the time factor should go away.
- Use a log scale, since votes tend to snowball, and so the first 10 votes are just as important as the next hundred.

## Reddit Sorting
[Reddit's comment sorting algorithm](https://medium.com/hacking-and-gonzo/how-reddit-ranking-algorithms-work-ef111e33d0d9), the wilson confidence sort, is inadequate, because it completely ignores time. What ends up happening, especially in smaller subreddits, is that the early comments end up getting upvoted, and newer comments stay at the bottom, never to be seen.

## Hacker News Sorting
The [Hacker New's ranking algorithm](https://medium.com/hacking-and-gonzo/how-hacker-news-ranking-algorithm-works-1d9b0cf2c08d) is great, but it doesn't use a log scale for the scores.

## My Algorithm
```
Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity

Score = Upvotes - Downvotes
Time = time since submission (in hours)
Gravity = Decay gravity, 1.8 is default
```

- Add 1 to the score, so that the standard new comment score of +1 will be affected by time decay. Otherwise all new comments would stay at zero, near the bottom.
- The sign and abs of the score are necessary for dealing with the log of negative scores.
- A scale factor of 10k gets the rank in integer form.

A plot of rank over 24 hours, of scores of 1, 5, 10, 100, 1000, with a scale factor of 10k.

![](https://i.imgur.com/w8oBLlL.png)
