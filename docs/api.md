# Lemmy WebSocket API
*Note: this may lag behind the actual API endpoints [here](../server/src/api).*

## Data types

- `i16`, `i32` and `i64` are respectively [16-bit](https://en.wikipedia.org/wiki/16-bit), [32-bit](https://en.wikipedia.org/wiki/32-bit) and [64-bit](https://en.wikipedia.org/wiki/64-bit_computing) integers.
- <code>Option<***SomeType***></code> designates an option which may be omitted in requests and not be present in responses. It will be of type ***SomeType***.
- <code>Vec<***SomeType***></code> is a list which contains objects of type ***SomeType***.
- `chrono::NaiveDateTime` is a timestamp string in [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601) format. Timestamps will be UTC.
- Other data types are listed [here](../server/src/db).

## Basic usage

Request and response strings are in [JSON format](https://www.json.org).

### Endpoint

Connect to <code>ws://***host***/api/v1/ws</code> to get started.

If the ***`host`*** supports secure connections, you can use <code>wss://***host***/api/v1/ws</code>.

### Testing with [Websocat](https://github.com/vi/websocat)
`websocat ws://127.0.0.1:8536/api/v1/ws -nt`

A simple test command:
`{"op": "ListCategories"}`

### Testing with the [WebSocket JavaScript API](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)
```javascript
var ws = new WebSocket("ws://" + host + "/api/v1/ws");
ws.onopen = function () {
  console.log("Connection succeed!");
  ws.send(JSON.stringify({
    op: "ListCategories"
  }));
};
```

## Rate limits

- 1 per hour for signups and community creation.
- 1 per 10 minutes for post creation.
- 30 actions per minute for post voting and comment creation.
- Everything else is not rate-limited.

## Errors
```rust
{
  op: String,
  message: String,
}
```

## API documentation

### Sort Types

These go wherever there is a `sort` field. The available sort types are:

- `Hot` - the hottest posts/communities, depending on votes, views, comments and publish date
- `New` - the newest posts/communities
- `TopDay` - the most upvoted posts/communities of the current day.
- `TopWeek` - the most upvoted posts/communities of the current week.
- `TopMonth` - the most upvoted posts/communities of the current month.
- `TopYear` - the most upvoted posts/communities of the current year.
- `TopAll` - the most upvoted posts/communities on the current instance.

### User / Authentication / Admin actions

#### Login

The `jwt` string should be stored and used anywhere `auth` is called for.

##### Request
```rust
{
  op: "Login",
  data: {
    username_or_email: String,
    password: String
  }
}
```
##### Response
```rust
{
  op: String,
  jwt: String
}
```


#### Register
Only the first user will be able to be the admin.

##### Request
```rust
{
  op: "Register",
  data: {
    username: String,
    email: Option<String>,
    password: String,
    password_verify: String,
    admin: bool
  }
}
```
##### Response
```rust
{
  op: String,
  jwt: String
}
```

#### Get User Details
##### Request
```rust
{
  op: "GetUserDetails",
  data: {
    user_id: Option<i32>,
    username: Option<String>,
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    community_id: Option<i32>,
    saved_only: bool,
    auth: Option<String>,
  }
}
```
##### Response
```rust
{
  op: String,
  user: UserView,
  follows: Vec<CommunityFollowerView>,
  moderates: Vec<CommunityModeratorView>,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
}
```
#### Save User Settings
##### Request
```rust
{
  op: "SaveUserSettings",
  data: {
    show_nsfw: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  jwt: String
}
```
#### Get Replies / Inbox
##### Request
```rust
{
  op: "GetReplies",
  data: {
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    unread_only: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  replies: Vec<ReplyView>,
}
```

#### Mark all replies as read
##### Request
```rust
{
  op: "MarkAllAsRead",
  data: {
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  replies: Vec<ReplyView>,
}
```

#### Add admin
##### Request
```rust
{
  op: "AddAdmin",
  data: {
    user_id: i32,
    added: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  admins: Vec<UserView>,
}
```

#### Ban user
##### Request
```rust
{
  op: "BanUser",
  data: {
    user_id: i32,
    ban: bool,
    reason: Option<String>,
    expires: Option<i64>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  user: UserView,
  banned: bool,
}
```

### Site
#### List Categories
##### Request
```rust
{
  op: "ListCategories"
}
```
##### Response
```rust
{
  op: String,
  categories: Vec<Category>
}
```

#### Search
Search types are `Both, Comments, Posts`.

##### Request
```rust
{
  op: "Search",
  data: {
    q: String,
    type_: String,
    community_id: Option<i32>,
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
  }
}
```
##### Response
```rust
{
  op: String,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
}
```

#### Get Modlog
##### Request
```rust
{
  op: "GetModlog",
  data: {
    mod_user_id: Option<i32>,
    community_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  }
}
```
##### Response
```rust
{
  op: String,
  removed_posts: Vec<ModRemovePostView>,
  locked_posts: Vec<ModLockPostView>,
  removed_comments: Vec<ModRemoveCommentView>,
  removed_communities: Vec<ModRemoveCommunityView>,
  banned_from_community: Vec<ModBanFromCommunityView>,
  banned: Vec<ModBanView>,
  added_to_community: Vec<ModAddCommunityView>,
  added: Vec<ModAddView>,
}
```

#### Create Site
##### Request
```rust
{
  op: "CreateSite",
  data: {
    name: String,
    description: Option<String>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  site: SiteView,
}
```

#### Edit Site
##### Request
```rust
{
  op: "EditSite",
  data: {
    name: String,
    description: Option<String>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  site: SiteView,
}
```

#### Get Site
##### Request
```rust
{
  op: "GetSite"
}
```
##### Response
```rust
{
  op: String,
  site: Option<SiteView>,
  admins: Vec<UserView>,
  banned: Vec<UserView>,
}
```

#### Transfer Site
##### Request
```rust
{
  op: "TransferSite",
  data: {
    user_id: i32,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  site: Option<SiteView>,
  admins: Vec<UserView>,
  banned: Vec<UserView>,
}
```

### Community
#### Get Community
##### Request
```rust
{
  op: "GetCommunity",
  data: {
    id: Option<i32>,
    name: Option<String>,
    auth: Option<String>
  }
}
```
##### Response
```rust
{
  op: String,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
}
```

#### Create Community
##### Request
```rust
{
  op: "CreateCommunity",
  data: {
    name: String,
    title: String,
    description: Option<String>,
    category_id: i32 ,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  community: CommunityView
}
```

#### List Communities
##### Request
```rust
{
  op: "ListCommunities",
  data: {
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    auth: Option<String>
  }
}
```
##### Response
```rust
{
  op: String,
  communities: Vec<CommunityView>
}
```

#### Ban from Community
##### Request
```rust
{
  op: "BanFromCommunity",
  data: {
    community_id: i32,
    user_id: i32,
    ban: bool,
    reason: Option<String>,
    expires: Option<i64>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  user: UserView,
  banned: bool,
}
```

#### Add Mod to Community
##### Request
```rust
{
  op: "AddModToCommunity",
  data: {
    community_id: i32,
    user_id: i32,
    added: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  moderators: Vec<CommunityModeratorView>,
}
```

#### Edit Community
Mods and admins can remove and lock a community, creators can delete it.

##### Request
```rust
{
  op: "EditCommunity",
  data: {
    edit_id: i32,
    name: String,
    title: String,
    description: Option<String>,
    category_id: i32,
    removed: Option<bool>,
    deleted: Option<bool>,
    reason: Option<String>,
    expires: Option<i64>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  community: CommunityView
}
```

#### Follow Community
##### Request
```rust
{
  op: "FollowCommunity",
  data: {
    community_id: i32,
    follow: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  community: CommunityView
}
```

#### Get Followed Communities
##### Request
```rust
{
  op: "GetFollowedCommunities",
  data: {
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  communities: Vec<CommunityFollowerView>
}
```

#### Transfer Community
##### Request
```rust
{
  op: "TransferCommunity",
  data: {
    community_id: i32,
    user_id: i32,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
}
```

### Post
#### Create Post
##### Request
```rust
{
  op: "CreatePost",
  data: {
    name: String,
    url: Option<String>,
    body: Option<String>,
    community_id: i32,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  post: PostView
}
```

#### Get Post
##### Request
```rust
{
  op: "GetPost",
  data: {
    id: i32,
    auth: Option<String>
  }
}
```
##### Response
```rust
{
  op: String,
  post: PostView,
  comments: Vec<CommentView>,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
}
```

#### Get Posts
Post listing types are `All, Subscribed, Community`

##### Request
```rust
{
  op: "GetPosts",
  data: {
    type_: String,
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    community_id: Option<i32>,
    auth: Option<String>
  }
}
```
##### Response
```rust
{
  op: String,
  posts: Vec<PostView>,
}
```

#### Create Post Like
`score` can be 0, -1, or 1

##### Request
```rust
{
  op: "CreatePostLike",
  data: {
    post_id: i32,
    score: i16,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  post: PostView
}
```

#### Edit Post
Mods and admins can remove and lock a post, creators can delete it.

##### Request
```rust
{
  op: "EditPost",
  data: {
    edit_id: i32,
    creator_id: i32,
    community_id: i32,
    name: String,
    url: Option<String>,
    body: Option<String>,
    removed: Option<bool>,
    deleted: Option<bool>,
    locked: Option<bool>,
    reason: Option<String>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  post: PostView
}
```

#### Save Post
##### Request
```rust
{
  op: "SavePost",
  data: {
    post_id: i32,
    save: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  post: PostView
}
```

### Comment
#### Create Comment
##### Request
```rust
{
  op: "CreateComment",
  data: {
    content: String,
    parent_id: Option<i32>,
    edit_id: Option<i32>,
    post_id: i32,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  comment: CommentView
}
```

#### Edit Comment
Mods and admins can remove a comment, creators can delete it.

##### Request
```rust
{
  op: "EditComment",
  data: {
    content: String,
    parent_id: Option<i32>,
    edit_id: i32,
    creator_id: i32,
    post_id: i32,
    removed: Option<bool>,
    deleted: Option<bool>,
    reason: Option<String>,
    read: Option<bool>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  comment: CommentView
}
```

#### Save Comment
##### Request
```rust
{
  op: "SaveComment",
  data: {
    comment_id: i32,
    save: bool,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  comment: CommentView
}
```

#### Create Comment Like
`score` can be 0, -1, or 1

##### Request
```rust
{
  op: "CreateCommentLike",
  data: {
    comment_id: i32,
    post_id: i32,
    score: i16,
    auth: String
  }
}
```
##### Response
```rust
{
  op: String,
  comment: CommentView
}
```
