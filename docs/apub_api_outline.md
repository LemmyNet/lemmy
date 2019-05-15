# Activitypub API outline

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
  "id": "https://instance_url/api/v1/user/sally_smith",
  "inbox": "https://instance_url/api/v1/user/sally_smith/inbox",
  "outbox": "https://instance_url/api/v1/user/sally_smith/outbox",
  "liked": "https://instance_url/api/v1/user/sally_smith/liked",
  // TODO disliked?
  "following": "https://instance_url/api/v1/user/sally_smith/following",
  "name": "sally_smith", 
  "preferredUsername": "Sally",
  "icon"?: {
    "type": "Image",
    "name": "User icon",
    "url": "https://instance_url/api/v1/user/sally_smith/icon.png",
    "width": 32,
    "height": 32
  },
  "published": "2014-12-31T23:00:00-08:00",
  "summary"?: "This is sally's profile."
}
```

### [Community / Group](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-group)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Group",
  "id": "https://instance_url/api/v1/community/today_i_learned",
  "name": "today_i_learned"
  "attributedTo": [ // The moderators
    "http://joe.example.org",
  ],
  "followers": "https://instance_url/api/v1/community/today_i_learned/followers",
  "published": "2014-12-31T23:00:00-08:00",
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
  "id": "https://instance_url/api/v1/post/1",
  "name": "The title of a post, maybe a link to imgur",
  "url": "https://news.blah.com"
  "attributedTo": "http://joe.example.org", // The poster
  "published": "2014-12-31T23:00:00-08:00",
}
```

### [Post Listings / Ordered CollectionPage](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-orderedcollectionpage)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "OrderedCollectionPage",
  "id": "https://instance_url/api/v1/posts?type={all, best, front}&sort={}&page=1,
  "partOf": "http://example.org/foo",
  "orderedItems": [Posts]
}
```

### [Comment / Note](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-note)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "id": "https://instance_url/api/v1/comment/1",
  "mediaType": "text/markdown",
  "content": "Looks like it is going to rain today. Bring an umbrella *if necessary*!"
  "attributedTo": john_id,
  "inReplyTo": "comment or post id",
  "published": "2014-12-31T23:00:00-08:00",
  "updated"?: "2014-12-12T12:12:12Z"
  "replies" // TODO, not sure if these objects should embed all replies in them or not.
  "to": [sally_id, group_id]
}
```
### [Comment Listings / Ordered CollectionPage](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-orderedcollectionpage)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "OrderedCollectionPage",
  "id": "https://instance_url/api/v1/comments?type={all,user,community,post,parent_comment}&id=1&page=1,
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
- These are all posts to a user's outbox.
- The server then creates a post to the necessary inbox of the recipient, or the followers.
- Whenever a user accesses the site, they do a get from their inbox.

### Comments
#### [Create](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-create)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "actor": id,
  "object": comment_id, or post_id
}
```
#### [Delete](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Delete",
  "actor": id,
  "object": comment_id, or post_id
}
```
#### [Update](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-update)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
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
  "type": "Read",
  "actor": user_id
  "object": comment_id
}
```

#### [Like](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-like)
- TODO: Should likes be notifications? IE, have a to?
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
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
  "type": "Create",
  "actor": id,
  "to": community_id/followers
  "object": post_id
}
```
#### [Delete](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Delete",
  "actor": id,
  "object": comment_id, or post_id
}
```

#### [Update](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-update)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
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
  "type": "Create",
  "actor": id,
  "object": community_id
}
```
#### [Delete](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-delete)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Delete",
  "actor": id,
  "object": community_id
}
```

#### [Update](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-update)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "actor": id,
  "object": community_id
  TODO fields.
}
```

#### [Follow / Subscribe](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-follow)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Follow",
  "actor": id
  "object": community_id
}
```

#### [Ignore/ Unsubscribe](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-ignore)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Follow",
  "actor": id
  "object": community_id
}
```
#### [Join / Become a Mod](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-join)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Join",
  "actor": user_id,
  "object": community_id
}
```

#### [Leave](https://www.w3.org/TR/activitystreams-vocabulary#dfn-leave)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
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
  "type": "Delete",
  "actor": id,
  "object": community_id
}
```

#### [Invite a moderator](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-invite)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Invite",
  "id": "https://instance_url/api/v1/invite/1",
  "actor": sally_id,
  "object": group_id,
  "target": john_id
}
```
#### [Accept Invitation](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-accept)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Accept",
  "actor": john_id,
  "object": invite_id
}
```
#### [Reject Invitation](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-reject)
```
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Reject",
  "actor": john_id,
  "object": invite_id
}
```

