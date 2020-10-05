# Lemmy API

*Note: this may lag behind the actual API endpoints [here](../src/api). The API should be considered unstable and may change any time.*

<!-- toc -->

- [Data types](#data-types)
- [Basic usage](#basic-usage)
  * [WebSocket](#websocket)
    + [Testing with Websocat](#testing-with-websocat)
    + [Testing with the WebSocket JavaScript API](#testing-with-the-websocket-javascript-api)
  * [HTTP](#http)
    + [Testing with Curl](#testing-with-curl)
      - [Get Example](#get-example)
      - [Post Example](#post-example)
- [Rate limits](#rate-limits)
- [Errors](#errors)
- [API documentation](#api-documentation)
  * [Sort Types](#sort-types)
  * [Undoing actions](#undoing-actions)
  * [Websocket vs HTTP](#websocket-vs-http)
  * [User / Authentication / Admin actions](#user--authentication--admin-actions)
    + [Login](#login)
      - [Request](#request)
      - [Response](#response)
      - [HTTP](#http-1)
    + [Register](#register)
      - [Request](#request-1)
      - [Response](#response-1)
      - [HTTP](#http-2)
    + [Get Captcha](#get-captcha)
      - [Request](#request-2)
      - [Response](#response-2)
      - [HTTP](#http-3)
    + [Get User Details](#get-user-details)
      - [Request](#request-3)
      - [Response](#response-3)
      - [HTTP](#http-4)
    + [Save User Settings](#save-user-settings)
      - [Request](#request-4)
      - [Response](#response-4)
      - [HTTP](#http-5)
    + [Get Replies / Inbox](#get-replies--inbox)
      - [Request](#request-5)
      - [Response](#response-5)
      - [HTTP](#http-6)
    + [Get User Mentions](#get-user-mentions)
      - [Request](#request-6)
      - [Response](#response-6)
      - [HTTP](#http-7)
    + [Mark User Mention as read](#mark-user-mention-as-read)
      - [Request](#request-7)
      - [Response](#response-7)
      - [HTTP](#http-8)
    + [Get Private Messages](#get-private-messages)
      - [Request](#request-8)
      - [Response](#response-8)
      - [HTTP](#http-9)
    + [Create Private Message](#create-private-message)
      - [Request](#request-9)
      - [Response](#response-9)
      - [HTTP](#http-10)
    + [Edit Private Message](#edit-private-message)
      - [Request](#request-10)
      - [Response](#response-10)
      - [HTTP](#http-11)
    + [Delete Private Message](#delete-private-message)
      - [Request](#request-11)
      - [Response](#response-11)
      - [HTTP](#http-12)
    + [Mark Private Message as Read](#mark-private-message-as-read)
      - [Request](#request-12)
      - [Response](#response-12)
      - [HTTP](#http-13)
    + [Mark All As Read](#mark-all-as-read)
      - [Request](#request-13)
      - [Response](#response-13)
      - [HTTP](#http-14)
    + [Delete Account](#delete-account)
      - [Request](#request-14)
      - [Response](#response-14)
      - [HTTP](#http-15)
    + [Add admin](#add-admin)
      - [Request](#request-15)
      - [Response](#response-15)
      - [HTTP](#http-16)
    + [Ban user](#ban-user)
      - [Request](#request-16)
      - [Response](#response-16)
      - [HTTP](#http-17)
    + [User Join](#user-join)
      - [Request](#request-17)
      - [Response](#response-17)
      - [HTTP](#http-18)
  * [Site](#site)
    + [List Categories](#list-categories)
      - [Request](#request-18)
      - [Response](#response-18)
      - [HTTP](#http-19)
    + [Search](#search)
      - [Request](#request-19)
      - [Response](#response-19)
      - [HTTP](#http-20)
    + [Get Modlog](#get-modlog)
      - [Request](#request-20)
      - [Response](#response-20)
      - [HTTP](#http-21)
    + [Create Site](#create-site)
      - [Request](#request-21)
      - [Response](#response-21)
      - [HTTP](#http-22)
    + [Edit Site](#edit-site)
      - [Request](#request-22)
      - [Response](#response-22)
      - [HTTP](#http-23)
    + [Get Site](#get-site)
      - [Request](#request-23)
      - [Response](#response-23)
      - [HTTP](#http-24)
    + [Transfer Site](#transfer-site)
      - [Request](#request-24)
      - [Response](#response-24)
      - [HTTP](#http-25)
    + [Get Site Config](#get-site-config)
      - [Request](#request-25)
      - [Response](#response-25)
      - [HTTP](#http-26)
    + [Save Site Config](#save-site-config)
      - [Request](#request-26)
      - [Response](#response-26)
      - [HTTP](#http-27)
  * [Community](#community)
    + [Get Community](#get-community)
      - [Request](#request-27)
      - [Response](#response-27)
      - [HTTP](#http-28)
    + [Create Community](#create-community)
      - [Request](#request-28)
      - [Response](#response-28)
      - [HTTP](#http-29)
    + [List Communities](#list-communities)
      - [Request](#request-29)
      - [Response](#response-29)
      - [HTTP](#http-30)
    + [Ban from Community](#ban-from-community)
      - [Request](#request-30)
      - [Response](#response-30)
      - [HTTP](#http-31)
    + [Add Mod to Community](#add-mod-to-community)
      - [Request](#request-31)
      - [Response](#response-31)
      - [HTTP](#http-32)
    + [Edit Community](#edit-community)
      - [Request](#request-32)
      - [Response](#response-32)
      - [HTTP](#http-33)
    + [Delete Community](#delete-community)
      - [Request](#request-33)
      - [Response](#response-33)
      - [HTTP](#http-34)
    + [Remove Community](#remove-community)
      - [Request](#request-34)
      - [Response](#response-34)
      - [HTTP](#http-35)
    + [Follow Community](#follow-community)
      - [Request](#request-35)
      - [Response](#response-35)
      - [HTTP](#http-36)
    + [Get Followed Communities](#get-followed-communities)
      - [Request](#request-36)
      - [Response](#response-36)
      - [HTTP](#http-37)
    + [Transfer Community](#transfer-community)
      - [Request](#request-37)
      - [Response](#response-37)
      - [HTTP](#http-38)
    + [Community Join](#community-join)
      - [Request](#request-38)
      - [Response](#response-38)
      - [HTTP](#http-39)
  * [Post](#post)
    + [Create Post](#create-post)
      - [Request](#request-39)
      - [Response](#response-39)
      - [HTTP](#http-40)
    + [Get Post](#get-post)
      - [Request](#request-40)
      - [Response](#response-40)
      - [HTTP](#http-41)
    + [Get Posts](#get-posts)
      - [Request](#request-41)
      - [Response](#response-41)
      - [HTTP](#http-42)
    + [Create Post Like](#create-post-like)
      - [Request](#request-42)
      - [Response](#response-42)
      - [HTTP](#http-43)
    + [Edit Post](#edit-post)
      - [Request](#request-43)
      - [Response](#response-43)
      - [HTTP](#http-44)
    + [Delete Post](#delete-post)
      - [Request](#request-44)
      - [Response](#response-44)
      - [HTTP](#http-45)
    + [Remove Post](#remove-post)
      - [Request](#request-45)
      - [Response](#response-45)
      - [HTTP](#http-46)
    + [Lock Post](#lock-post)
      - [Request](#request-46)
      - [Response](#response-46)
      - [HTTP](#http-47)
    + [Sticky Post](#sticky-post)
      - [Request](#request-47)
      - [Response](#response-47)
      - [HTTP](#http-48)
    + [Save Post](#save-post)
      - [Request](#request-48)
      - [Response](#response-48)
      - [HTTP](#http-49)
    + [Post Join](#post-join)
      - [Request](#request-49)
      - [Response](#response-49)
      - [HTTP](#http-50)
  * [Comment](#comment)
    + [Create Comment](#create-comment)
      - [Request](#request-50)
      - [Response](#response-50)
      - [HTTP](#http-51)
    + [Edit Comment](#edit-comment)
      - [Request](#request-51)
      - [Response](#response-51)
      - [HTTP](#http-52)
    + [Delete Comment](#delete-comment)
      - [Request](#request-52)
      - [Response](#response-52)
      - [HTTP](#http-53)
    + [Remove Comment](#remove-comment)
      - [Request](#request-53)
      - [Response](#response-53)
      - [HTTP](#http-54)
    + [Get Comments](#get-comments)
      - [Request](#request-54)
      - [Response](#response-54)
      - [HTTP](#http-55)
    + [Mark Comment as Read](#mark-comment-as-read)
      - [Request](#request-55)
      - [Response](#response-55)
      - [HTTP](#http-56)
    + [Save Comment](#save-comment)
      - [Request](#request-56)
      - [Response](#response-56)
      - [HTTP](#http-57)
    + [Create Comment Like](#create-comment-like)
      - [Request](#request-57)
      - [Response](#response-57)
      - [HTTP](#http-58)
  * [RSS / Atom feeds](#rss--atom-feeds)
    + [All](#all)
    + [Community](#community-1)
    + [User](#user)

<!-- tocstop -->

## Data types

- `i16`, `i32` and `i64` are respectively [16-bit](https://en.wikipedia.org/wiki/16-bit), [32-bit](https://en.wikipedia.org/wiki/32-bit) and [64-bit](https://en.wikipedia.org/wiki/64-bit_computing) integers.
- <code>Option<***SomeType***></code> designates an option which may be omitted in requests and not be present in responses. It will be of type ***SomeType***.
- <code>Vec<***SomeType***></code> is a list which contains objects of type ***SomeType***.
- `chrono::NaiveDateTime` is a timestamp string in [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601) format. Timestamps will be UTC.
- Other data types are listed [here](../src/db).

## Basic usage

Request and response strings are in [JSON format](https://www.json.org).

### WebSocket

Connect to <code>ws://***host***/api/v1/ws</code> to get started.

If the ***`host`*** supports secure connections, you can use <code>wss://***host***/api/v1/ws</code>.

To receive websocket messages, you must join a room / context. The three available are:

- [UserJoin](#user-join). Receives replies, private messages, etc.
- [PostJoin](#post-join). Receives new comments on a post.
- [CommunityJoin](#community-join). Receives front page / community posts.

#### Testing with Websocat

[Websocat link](https://github.com/vi/websocat)

`websocat ws://127.0.0.1:8536/api/v1/ws -nt`

A simple test command:
`{"op": "ListCategories"}`

#### Testing with the WebSocket JavaScript API

[WebSocket JavaScript API](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)
```javascript
var ws = new WebSocket("ws://" + host + "/api/v1/ws");
ws.onopen = function () {
  console.log("Connection succeed!");
  ws.send(JSON.stringify({
    op: "ListCategories"
  }));
};
```
### HTTP

Endpoints are at <code>http://***host***/api/v1/***endpoint***</code>. They'll be listed below for each action.

#### Testing with Curl

##### Get Example

```
curl /community/list?sort=Hot
```

##### Post Example

```
curl -i -H \
"Content-Type: application/json" \
-X POST \
-d '{
  "comment_id": X,
  "post_id": X,
  "score": X,
  "auth": "..."
}' \
/comment/like
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

- `Active` - the hottest posts/communities, depending on votes, and newest comment publish date.
- `Hot` - the hottest posts/communities, depending on votes and publish date.
- `New` - the newest posts/communities
- `TopDay` - the most upvoted posts/communities of the current day.
- `TopWeek` - the most upvoted posts/communities of the current week.
- `TopMonth` - the most upvoted posts/communities of the current month.
- `TopYear` - the most upvoted posts/communities of the current year.
- `TopAll` - the most upvoted posts/communities on the current instance.

### Undoing actions

Whenever you see a `deleted: bool`, `removed: bool`, `read: bool`, `locked: bool`, etc, you can undo this action by sending `false`.

### Websocket vs HTTP

- Below are the websocket JSON requests / responses. For HTTP, ignore all fields except those inside `data`.
- For example, an http login will be a `POST` `{username_or_email: X, password: X}`

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
  op: "Login",
  data: {
    jwt: String,
  }
}
```

##### HTTP

`POST /user/login`

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
    admin: bool,
    captcha_uuid: Option<String>, // Only checked if these are enabled in the server
    captcha_answer: Option<String>,
  }
}
```
##### Response
```rust
{
  op: "Register",
  data: {
    jwt: String,
  }
}
```

##### HTTP

`POST /user/register`

#### Get Captcha

These expire after 10 minutes.

##### Request
```rust
{
  op: "GetCaptcha",
}
```
##### Response
```rust
{
  op: "GetCaptcha",
  data: {
    ok?: { // Will be undefined if captchas are disabled
      png: String, // A Base64 encoded png
      wav: Option<String>, // A Base64 encoded wav audio file
      uuid: String,
    }
  }
}
```

##### HTTP

`GET /user/get_captcha`

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
  op: "GetUserDetails",
  data: {
    user: UserView,
    follows: Vec<CommunityFollowerView>,
    moderates: Vec<CommunityModeratorView>,
    comments: Vec<CommentView>,
    posts: Vec<PostView>,
  }
}
```
##### HTTP

`GET /user`

#### Save User Settings
##### Request
```rust
{
  op: "SaveUserSettings",
  data: {
    show_nsfw: bool,
    theme: String, // Default 'darkly'
    default_sort_type: i16, // The Sort types from above, zero indexed as a number
    default_listing_type: i16, // Post listing types are `All, Subscribed, Community`
    lang: String,
    avatar: Option<String>,
    banner: Option<String>,
    preferred_username: Option<String>,
    email: Option<String>,
    bio: Option<String>,
    matrix_user_id: Option<String>,
    new_password: Option<String>,
    new_password_verify: Option<String>,
    old_password: Option<String>,
    show_avatars: bool,
    send_notifications_to_email: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "SaveUserSettings",
  data: {
    jwt: String
  }
}
```
##### HTTP

`PUT /user/save_user_settings`

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
  op: "GetReplies",
  data: {
    replies: Vec<ReplyView>,
  }
}
```
##### HTTP

`GET /user/replies`


#### Get User Mentions
##### Request
```rust
{
  op: "GetUserMentions",
  data: {
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    unread_only: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "GetUserMentions",
  data: {
    mentions: Vec<UserMentionView>,
  }
}
```

##### HTTP

`GET /user/mention`

#### Mark User Mention as read

Only the recipient can do this.

##### Request
```rust
{
  op: "MarkUserMentionAsRead",
  data: {
    user_mention_id: i32,
    read: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "MarkUserMentionAsRead",
  data: {
    mention: UserMentionView,
  }
}
```
##### HTTP

`POST /user/mention/mark_as_read`

#### Get Private Messages
##### Request
```rust
{
  op: "GetPrivateMessages",
  data: {
    unread_only: bool,
    page: Option<i64>,
    limit: Option<i64>,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "GetPrivateMessages",
  data: {
    messages: Vec<PrivateMessageView>,
  }
}
```

##### HTTP

`GET /private_message/list`

#### Create Private Message
##### Request
```rust
{
  op: "CreatePrivateMessage",
  data: {
    content: String,
    recipient_id: i32,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "CreatePrivateMessage",
  data: {
    message: PrivateMessageView,
  }
}
```

##### HTTP

`POST /private_message`

#### Edit Private Message
##### Request
```rust
{
  op: "EditPrivateMessage",
  data: {
    edit_id: i32,
    content: String,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "EditPrivateMessage",
  data: {
    message: PrivateMessageView,
  }
}
```

##### HTTP

`PUT /private_message`

#### Delete Private Message
##### Request
```rust
{
  op: "DeletePrivateMessage",
  data: {
    edit_id: i32,
    deleted: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "DeletePrivateMessage",
  data: {
    message: PrivateMessageView,
  }
}
```

##### HTTP

`POST /private_message/delete`

#### Mark Private Message as Read

Only the recipient can do this.

##### Request
```rust
{
  op: "MarkPrivateMessageAsRead",
  data: {
    edit_id: i32,
    read: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "MarkPrivateMessageAsRead",
  data: {
    message: PrivateMessageView,
  }
}
```

##### HTTP

`POST /private_message/mark_as_read`

#### Mark All As Read

Marks all user replies and mentions as read.

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
  op: "MarkAllAsRead",
  data: {
    replies: Vec<ReplyView>,
  }
}
```

##### HTTP

`POST /user/mark_all_as_read`

#### Delete Account

*Permanently deletes your posts and comments*

##### Request
```rust
{
  op: "DeleteAccount",
  data: {
    password: String,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "DeleteAccount",
  data: {
    jwt: String,
  }
}
```

##### HTTP

`POST /user/delete_account`

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
  op: "AddAdmin",
  data: {
    admins: Vec<UserView>,
  }
}
```
##### HTTP

`POST /admin/add`

#### Ban user
##### Request
```rust
{
  op: "BanUser",
  data: {
    user_id: i32,
    ban: bool,
    remove_data: Option<bool>, // Removes/Restores their comments, posts, and communities
    reason: Option<String>,
    expires: Option<i64>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "BanUser",
  data: {
    user: UserView,
    banned: bool,
  }
}
```
##### HTTP

`POST /user/ban`

#### User Join
##### Request
```rust
{
  op: "UserJoin",
  data: {
    auth: String
  }
}
```
##### Response
```rust
{
  op: "UserJoin",
  data: {
    joined: bool,
  }
}
```
##### HTTP

`POST /user/join`

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
  op: "ListCategories",
  data: {
    categories: Vec<Category>
  }
}
```
##### HTTP

`GET /categories`

#### Search

Search types are `All, Comments, Posts, Communities, Users, Url`

##### Request
```rust
{
  op: "Search",
  data: {
    q: String,
    type_: String,
    community_id: Option<i32>,
    community_name: Option<String>,
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    auth?: Option<String>,
  }
}
```
##### Response
```rust
{
  op: "Search",
  data: {
    type_: String,
    comments: Vec<CommentView>,
    posts: Vec<PostView>,
    communities: Vec<CommunityView>,
    users: Vec<UserView>,
  }
}
```
##### HTTP

`GET /search`

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
  op: "GetModlog",
  data: {
    removed_posts: Vec<ModRemovePostView>,
    locked_posts: Vec<ModLockPostView>,
    removed_comments: Vec<ModRemoveCommentView>,
    removed_communities: Vec<ModRemoveCommunityView>,
    banned_from_community: Vec<ModBanFromCommunityView>,
    banned: Vec<ModBanView>,
    added_to_community: Vec<ModAddCommunityView>,
    added: Vec<ModAddView>,
  }
}
```

##### HTTP

`GET /modlog`

#### Create Site
##### Request
```rust
{
  op: "CreateSite",
  data: {
    name: String,
    description: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "CreateSite",
    data: {
    site: SiteView,
  }
}
```

##### HTTP

`POST /site`

#### Edit Site
##### Request
```rust
{
  op: "EditSite",
  data: {
    name: String,
    description: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "EditSite",
  data: {
    site: SiteView,
  }
}
```
##### HTTP

`PUT /site`

#### Get Site
##### Request
```rust
{
  op: "GetSite"
  data: {
    auth: Option<String>,
  }

}
```
##### Response
```rust
{
  op: "GetSite",
  data: {
    site: Option<SiteView>,
    admins: Vec<UserView>,
    banned: Vec<UserView>,
    online: usize, // This is currently broken
    version: String,
    my_user: Option<User_>, // Gives back your user and settings if logged in
  }
}
```
##### HTTP

`GET /site`

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
  op: "TransferSite",
  data: {
    site: Option<SiteView>,
    admins: Vec<UserView>,
    banned: Vec<UserView>,
  }
}
```
##### HTTP

`POST /site/transfer`

#### Get Site Config
##### Request
```rust
{
  op: "GetSiteConfig",
  data: {
    auth: String
  }
}
```
##### Response
```rust
{
  op: "GetSiteConfig",
  data: {
    config_hjson: String,
  }
}
```
##### HTTP

`GET /site/config`

#### Save Site Config
##### Request
```rust
{
  op: "SaveSiteConfig",
  data: {
    config_hjson: String,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "SaveSiteConfig",
  data: {
    config_hjson: String,
  }
}
```
##### HTTP

`PUT /site/config`

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
  op: "GetCommunity",
  data: {
    community: CommunityView,
    moderators: Vec<CommunityModeratorView>,
  }
}
```
##### HTTP

`GET /community`

#### Create Community
##### Request
```rust
{
  op: "CreateCommunity",
  data: {
    name: String,
    title: String,
    description: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    category_id: i32 ,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "CreateCommunity",
  data: {
    community: CommunityView
  }
}
```
##### HTTP

`POST /community`

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
  op: "ListCommunities",
  data: {
    communities: Vec<CommunityView>
  }
}
```
##### HTTP

`GET /community/list`

#### Ban from Community
##### Request
```rust
{
  op: "BanFromCommunity",
  data: {
    community_id: i32,
    user_id: i32,
    ban: bool,
    remove_data: Option<bool>, // Removes/Restores their comments and posts for that community
    reason: Option<String>,
    expires: Option<i64>,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "BanFromCommunity",
  data: {
    user: UserView,
    banned: bool,
  }
}
```
##### HTTP

`POST /community/ban_user`

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
  op: "AddModToCommunity",
  data: {
    moderators: Vec<CommunityModeratorView>,
  }
}
```
##### HTTP

`POST /community/mod`

#### Edit Community
Only mods can edit a community.

##### Request
```rust
{
  op: "EditCommunity",
  data: {
    edit_id: i32,
    title: String,
    description: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    category_id: i32,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "EditCommunity",
  data: {
    community: CommunityView
  }
}
```
##### HTTP

`PUT /community`

#### Delete Community
Only a creator can delete a community

##### Request
```rust
{
  op: "DeleteCommunity",
  data: {
    edit_id: i32,
    deleted: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "DeleteCommunity",
  data: {
    community: CommunityView
  }
}
```
##### HTTP

`POST /community/delete`

#### Remove Community
Only admins can remove a community.

##### Request
```rust
{
  op: "RemoveCommunity",
  data: {
    edit_id: i32,
    removed: bool,
    reason: Option<String>,
    expires: Option<i64>,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "RemoveCommunity",
  data: {
    community: CommunityView
  }
}
```
##### HTTP

`POST /community/remove`

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
  op: "FollowCommunity",
  data: {
    community: CommunityView
  }
}
```
##### HTTP

`POST /community/follow`

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
  op: "GetFollowedCommunities",
  data: {
    communities: Vec<CommunityFollowerView>
  }
}
```
##### HTTP

`GET /user/followed_communities`

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
  op: "TransferCommunity",
  data: {
    community: CommunityView,
    moderators: Vec<CommunityModeratorView>,
    admins: Vec<UserView>,
  }
}
```
##### HTTP

`POST /community/transfer`

#### Community Join

The main / frontpage community is `community_id: 0`.

##### Request
```rust
{
  op: "CommunityJoin",
  data: {
    community_id: i32
  }
}
```
##### Response
```rust
{
  op: "CommunityJoin",
  data: {
    joined: bool,
  }
}
```
##### HTTP

`POST /community/join`

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
    nsfw: bool,
    community_id: i32,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "CreatePost",
  data: {
    post: PostView
  }
}
```
##### HTTP

`POST /post`

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
  op: "GetPost",
  data: {
    post: PostView,
    comments: Vec<CommentView>,
    community: CommunityView,
    moderators: Vec<CommunityModeratorView>,
  }
}
```
##### HTTP

`GET /post`

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
    community_name: Option<String>,
    auth: Option<String>
  }
}
```
##### Response
```rust
{
  op: "GetPosts",
  data: {
    posts: Vec<PostView>,
  }
}
```
##### HTTP

`GET /post/list`

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
  op: "CreatePostLike",
  data: {
    post: PostView
  }
}
```
##### HTTP

`POST /post/like`

#### Edit Post
##### Request
```rust
{
  op: "EditPost",
  data: {
    edit_id: i32,
    name: String,
    url: Option<String>,
    body: Option<String>,
    nsfw: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "EditPost",
  data: {
    post: PostView
  }
}
```

##### HTTP

`PUT /post`

#### Delete Post
##### Request
```rust
{
  op: "DeletePost",
  data: {
    edit_id: i32,
    deleted: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "DeletePost",
  data: {
    post: PostView
  }
}
```

##### HTTP

`POST /post/delete`

#### Remove Post

Only admins and mods can remove a post.

##### Request
```rust
{
  op: "RemovePost",
  data: {
    edit_id: i32,
    removed: bool,
    reason: Option<String>,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "RemovePost",
  data: {
    post: PostView
  }
}
```

##### HTTP

`POST /post/remove`

#### Lock Post

Only admins and mods can lock a post.

##### Request
```rust
{
  op: "LockPost",
  data: {
    edit_id: i32,
    locked: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "LockPost",
  data: {
    post: PostView
  }
}
```

##### HTTP

`POST /post/lock`

#### Sticky Post

Only admins and mods can sticky a post.

##### Request
```rust
{
  op: "StickyPost",
  data: {
    edit_id: i32,
    stickied: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "StickyPost",
  data: {
    post: PostView
  }
}
```

##### HTTP

`POST /post/sticky`

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
  op: "SavePost",
  data: {
    post: PostView
  }
}
```
##### HTTP

`POST /post/save`

#### Post Join
##### Request
```rust
{
  op: "PostJoin",
  data: {
    post_id: i32
  }
}
```
##### Response
```rust
{
  op: "PostJoin",
  data: {
    joined: bool,
  }
}
```
##### HTTP

`POST /post/join`

### Comment
#### Create Comment
##### Request
```rust
{
  op: "CreateComment",
  data: {
    content: String,
    parent_id: Option<i32>,
    post_id: i32,
    form_id: Option<String>, // An optional form id, so you know which message came back
    auth: String
  }
}
```
##### Response
```rust
{
  op: "CreateComment",
  data: {
    comment: CommentView
  }
}
```

##### HTTP

`POST /comment`

#### Edit Comment

Only the creator can edit the comment.

##### Request
```rust
{
  op: "EditComment",
  data: {
    content: String,
    edit_id: i32,
    form_id: Option<String>,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "EditComment",
  data: {
    comment: CommentView
  }
}
```
##### HTTP

`PUT /comment`

#### Delete Comment

Only the creator can delete the comment.

##### Request
```rust
{
  op: "DeleteComment",
  data: {
    edit_id: i32,
    deleted: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "DeleteComment",
  data: {
    comment: CommentView
  }
}
```
##### HTTP

`POST /comment/delete`


#### Remove Comment

Only a mod or admin can remove the comment.

##### Request
```rust
{
  op: "RemoveComment",
  data: {
    edit_id: i32,
    removed: bool,
    reason: Option<String>,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "RemoveComment",
  data: {
    comment: CommentView
  }
}
```
##### HTTP

`POST /comment/remove`

#### Get Comments

Comment listing types are `All, Subscribed, Community`

##### Request
```rust
{
  op: "GetComments",
  data: {
    type_: String,
    sort: String,
    page: Option<i64>,
    limit: Option<i64>,
    community_id: Option<i32>,
    community_name: Option<String>,
    auth: Option<String>
  }
}
```
##### Response
```rust
{
  op: "GetComments",
  data: {
    comments: Vec<CommentView>,
  }
}
```
##### HTTP

`GET /comment/list`

#### Mark Comment as Read

Only the recipient can do this.

##### Request
```rust
{
  op: "MarkCommentAsRead",
  data: {
    edit_id: i32,
    read: bool,
    auth: String,
  }
}
```
##### Response
```rust
{
  op: "MarkCommentAsRead",
  data: {
    comment: CommentView
  }
}
```
##### HTTP

`POST /comment/mark_as_read`

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
  op: "SaveComment",
  data: {
    comment: CommentView
  }
}
```
##### HTTP

`PUT /comment/save`

#### Create Comment Like

`score` can be 0, -1, or 1

##### Request
```rust
{
  op: "CreateCommentLike",
  data: {
    comment_id: i32,
    score: i16,
    auth: String
  }
}
```
##### Response
```rust
{
  op: "CreateCommentLike",
  data: {
    comment: CommentView
  }
}
```
##### HTTP

`POST /comment/like`

### RSS / Atom feeds

#### All

`/feeds/all.xml?sort=Hot`

#### Community

`/feeds/c/community-name.xml?sort=Hot`

#### User

`/feeds/u/user-name.xml?sort=Hot`

