# API

- Start with the [reddit API](https://www.reddit.com/dev/api), and find [Activitypub vocab](https://www.w3.org/TR/activitystreams-vocabulary/) to match it.

<!-- toc -->

- [Actors](#actors)
  * [User / Person](#user--person)
  * [Community / Group](#community--group)
- [Objects](#objects)
  * [Post / Page](#post--page)
  * [Post Listings / Ordered CollectionPage](#post-listings--ordered-collectionpage)
  * [Comment / Note](#comment--note)
  * [Comment Listings / Ordered CollectionPage](#comment-listings--ordered-collectionpage)
  * [Deleted thing / Tombstone](#deleted-thing--tombstone)
- [Actions](#actions)
  * [Comments](#comments)
    + [Create](#create)
    + [Delete](#delete)
    + [Update](#update)
    + [Read](#read)
    + [Like](#like)
    + [Dislike](#dislike)
  * [Posts](#posts)
    + [Create](#create-1)
    + [Delete](#delete-1)
    + [Update](#update-1)
    + [Read](#read-1)
  * [Communities](#communities)
    + [Create](#create-2)
    + [Delete](#delete-2)
    + [Update](#update-2)
    + [Join](#join)
    + [Leave](#leave)
  * [Moderator](#moderator)
    + [Ban user from community / Block](#ban-user-from-community--block)
    + [Delete Comment](#delete-comment)
    + [Invite a moderator](#invite-a-moderator)
    + [Accept Invitation](#accept-invitation)
    + [Reject Invitation](#reject-invitation)

<!-- tocstop -->

## Actors

### [User / Person](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-person)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Person",
  "id": "https://rust-reddit-fediverse/api/v1/user/sally_smith",
  "name": "Sally Smith", // Their chosen alias
  "icon"?: {
    "type": "Image",
    "name": "User icon",
    "url": "https://rust-reddit-fediverse/api/v1/user/sally_smith/icon.png",
    "width": 32,
    "height": 32
  },
  "startTime": "2014-12-31T23:00:00-08:00",
  "summary"?: "This is sally's profile."
}
```

### [Community / Group](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-group)

```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Group",
  "id": "https://rust-reddit-fediverse/api/v1/community/today_i_learned",
  "name": "today_i_learned"
  "attributedTo": [ // The moderators
    "http://joe.example.org",
  ],
  "startTime": "2014-12-31T23:00:00-08:00",
  "summary"?: "The group's tagline",
  "attachment: [{}] // TBD, these would be where strong types for custom styles, and images would work.
}
```

## Objects

### [Post / Page](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-page) 
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Page",
  "id": "https://rust-reddit-fediverse/api/v1/post/1",
  "name": "The title of a post, maybe a link to imgur",
  "url": "https://news.blah.com"
  "attributedTo": "http://joe.example.org", // The poster
  "startTime": "2014-12-31T23:00:00-08:00",

}
```

### [Post Listings / Ordered CollectionPage](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-orderedcollectionpage)

```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Page 1 of Sally's front page",
  "type": "OrderedCollectionPage",
  "id": "https://rust-reddit-fediverse/api/v1/posts?type={all, best, front}&sort={}&page=1,
  "partOf": "http://example.org/foo",
  "orderedItems": [Posts]
}
```

### [Comment / Note](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-note)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "id": "https://rust-reddit-fediverse/api/v1/comment/1",
  "name": "A note",
  "content": "Looks like it is going to rain today. Bring an umbrella <a href='http://sally.example.org'>@sally</a>!"
  "attributedTo": john_id,
  "inReplyTo": "comment or post id",
  "startTime": "2014-12-31T23:00:00-08:00",
  "updated"?: "2014-12-12T12:12:12Z"
  "replies" // TODO, not sure if these objects should embed all replies in them or not.
  "to": [sally_id, group_id]
}
```
### [Comment Listings / Ordered CollectionPage](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-orderedcollectionpage)

```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Page 1 of comments for",
  "type": "OrderedCollectionPage",
  "id": "https://rust-reddit-fediverse/api/v1/comments?type={all,user,community,post,parent_comment}&id=1&page=1,
  "partOf": "http://example.org/foo",
  "orderedItems": [Comments]
}
```
### [Deleted thing / Tombstone](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-tombstone)
```
{
  "type": "Tombstone",
  "formerType": "Note / Post",
  "id": note / post_id,
  "deleted": "2016-03-17T00:00:00Z"
}
```
## Actions

### Comments

#### [Create](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-create)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally created a note",
  "type": "Create",
  "actor": id,
  "object": comment_id, or post_id
}
```

#### [Delete](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally deleted a note",
  "type": "Delete",
  "actor": id,
  "object": comment_id, or post_id
}
```
#### [Update](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-update)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally created a note",
  "type": "Create",
  "actor": id,
  "object": comment_id, or post_id
  "content": "New comment",
  "updated": "New Date"
}
```
#### [Read](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-read)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally read a comment",
  "type": "Read",
  "actor": user_id
  "object": comment_id
}
```

#### [Like](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-like)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally liked a comment",
  "type": "Like",
  "actor": user_id
  "object": comment_id
  // TODO different types of reactions, or no?
}
```
#### [Dislike](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-dislike)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally disliked a comment",
  "type": "Dislike",
  "actor": user_id
  "object": comment_id
  // TODO different types of reactions, or no?
}
```

### Posts
#### [Create](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-create)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally created a post",
  "type": "Create",
  "actor": id,
  "object": post_id
}
```
#### [Delete](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally deleted a post",
  "type": "Delete",
  "actor": id,
  "object": comment_id, or post_id
}
```

#### [Update](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-update)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally created a post",
  "type": "Create",
  "actor": id,
  "object": comment_id, or post_id
  TODO fields.
}
```
#### [Read](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-read)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally read a post",
  "type": "Read",
  "actor": user_id
  "object": post_id
}
```

### Communities
#### [Create](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-create)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally created a community",
  "type": "Create",
  "actor": id,
  "object": community_id
}
```
#### [Delete](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally deleted a community",
  "type": "Delete",
  "actor": id,
  "object": community_id
}
```

#### [Update](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-update)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally created a community",
  "type": "Create",
  "actor": id,
  "object": community_id
  TODO fields.
}
```

#### [Join](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-join)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally joined a community",
  "type": "Join",
  "actor": user_id,
  "object": community_id
}
```

#### [Leave](https://www.w3.org/TR/activitystreams-vocabulary#dfn-leave)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally left a community",
  "type": "Leave",
  "actor": user_id,
  "object": community_id
}
```

### Moderator
#### [Ban user from community / Block](https://www.w3.org/TR/activitystreams-vocabulary#dfn-block)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "The moderator blocked Sally from a group",
  "type": "Remove",
  "actor": mod_id,
  "object": user_id,
  "origin": group_id
}
```

#### [Delete Comment](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally deleted a users comment",
  "type": "Delete",
  "actor": id,
  "object": community_id
}
```

#### [Invite a moderator](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-invite)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "Sally invited John to mod a community",
  "type": "Invite",
  "id": "https://rust-reddit-fediverse/api/v1/invite/1",
  "actor": sally_id,
  "object": group_id,
  "target": john_id
}
```
#### [Accept Invitation](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-accept)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "John Accepted an invitation to mod a community",
  "type": "Accept",
  "actor": john_id,
  "object": invite_id
}
```
#### [Reject Invitation](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-reject)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "summary": "John Rejected an invitation to mod a community",
  "type": "Reject",
  "actor": john_id,
  "object": invite_id
}
```

