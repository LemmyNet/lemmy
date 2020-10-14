# Activitypub API outline

This document is targeted at developers who are familiar with the ActivityPub and ActivityStreams protocols. It gives a detailed outline of the actors, objects and activities used by Lemmy.

Before reading this, have a look at our [Federation Overview](contributing_federation_overview.md) to get an idea how Lemmy federation works on a high level.

Lemmy does not yet follow the ActivityPub spec in all regards. For example, we don't set a valid context indicating our context fields. We also ignore fields like `inbox`, `outbox` or `endpoints` for remote actors, and assume that everything is Lemmy. For an overview of deviations, read [#698](https://github.com/LemmyNet/lemmy/issues/698). They will be fixed in the near future.

Lemmy is also really inflexible when it comes to incoming activities and objects. They need to be exactly identical to the examples below. Things like having an array instead of a single value, or an object ID instead of the full object will result in an error.

In the following tables, "mandatory" refers to whether or not Lemmy will accept an incoming activity without this field. Lemmy itself will always include all non-empty fields.

<!-- toc -->

- [Actors](#actors)
  * [Community](#community)
  * [User](#user)
- [Objects](#objects)
  * [Post](#post)
  * [Comment](#comment)
  * [Private Message](#private-message)
- [Activities](#activities)
  * [Follow](#follow)
  * [Accept Follow](#accept-follow)
  * [Unfollow](#unfollow)
  * [Create or Update Post](#create-or-update-post)
  * [Create or Update Comment](#create-or-update-comment)
  * [Like Post or Comment](#like-post-or-comment)
  * [Dislike Post or Comment](#dislike-post-or-comment)
  * [Delete Post or Comment](#delete-post-or-comment)
  * [Remove Post or Comment](#remove-post-or-comment)
  * [Undo](#undo)
  * [Announce](#announce)
  * [Create or Update Private message](#create-or-update-private-message)
  * [Delete Private Message](#delete-private-message)
  * [Undo Delete Private Message](#undo-delete-private-message)

<!-- tocstop -->

## Actors

### Community

An automated actor. Users can send posts or comments to it, which the community forwards to its followers in the form of `Announce`.

Sends activities to user: `Accept/Follow`, `Announce`

Receives activities from user: `Follow`, `Undo/Follow`, `Create`, `Update`, `Like`, `Dislike`, `Remove` (only admin/mod), `Delete` (only creator), `Undo` (only for own actions)

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/c/main",
    "type": "Group",
    "name": "main",
    "preferredUsername": "The Main Community",
    "category": { 
        "identifier": "1",
        "name": "Discussion"
    },
    "sensitive": false,
    "attributedTo": [
        "https://enterprise.lemmy.ml/u/picard",
        "https://enterprise.lemmy.ml/u/riker"
    ],
    "content": "Welcome to the default community!",
    "icon": {
        "type": "Image",
        "url": "https://enterprise.lemmy.ml/pictrs/image/Z8pFFb21cl.png"
    },
    "image": {
        "type": "Image",
        "url": "https://enterprise.lemmy.ml/pictrs/image/Wt8zoMcCmE.jpg"
    },
    "inbox": "https://enterprise.lemmy.ml/c/main/inbox",
    "outbox": "https://enterprise.lemmy.ml/c/main/outbox",
    "followers": "https://enterprise.lemmy.ml/c/main/followers",
    "endpoints": {
        "sharedInbox": "https://enterprise.lemmy.ml/inbox"
    },
    "published": "2020-10-06T17:27:43.282386+00:00",
    "updated": "2020-10-08T11:57:50.545821+00:00",
    "publicKey": {
        "id": "https://enterprise.lemmy.ml/c/main#main-key",
        "owner": "https://enterprise.lemmy.ml/c/main",
        "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA9JJ7Ybp/H7iXeLkWFepg\ny4PHyIXY1TO9rK3lIBmAjNnkNywyGXMgUiiVhGyN9yU7Km8aWayQsNHOkPL7wMZK\nnY2Q+CTQv49kprEdcDVPGABi6EbCSOcRFVaUjjvRHf9Olod2QP/9OtX0oIFKN2KN\nPIUjeKK5tw4EWB8N1i5HOuOjuTcl2BXSemCQLAlXerLjT8xCarGi21xHPaQvAuns\nHt8ye7fUZKPRT10kwDMapjQ9Tsd+9HeBvNa4SDjJX1ONskNh2j4bqHHs2WUymLpX\n1cgf2jmaXAsz6jD9u0wfrLPelPJog8RSuvOzDPrtwX6uyQOl5NK00RlBZwj7bMDx\nzwIDAQAB\n-----END PUBLIC KEY-----\n"
    }
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `name` | yes | Name of the actor |
| `preferredUsername` | yes | Displayname |
| `category` | yes | Hardcoded list of categories, see https://dev.lemmy.ml/api/v1/categories |
| `sensitive` | yes | True indicates that all posts in the community are nsfw |
| `attributedTo` | yes | First the community creator, then all the remaining moderators |
| `content` | no | Text for the community sidebar, usually containing a description and rules |
| `icon` | no | Icon, shown next to the community name |
| `image` | no | Banner image, shown on top of the community page |
| `inbox` | no | ActivityPub inbox URL |
| `outbox` | no | ActivityPub outbox URL, only contains up to 20 latest posts, no comments, votes or other activities |
| `followers` | no | Follower collection URL, only contains the number of followers, no references to individual followers |
| `endpoints` | no | Contains URL of shared inbox |
| `published` | no | Datetime when the community was first created |
| `updated` | no | Datetime when the community was last changed |
| `publicKey` | yes | The public key used to verify signatures from this actor |
   
### User

A person, interacts primarily with the community where it sends and receives posts/comments. Can also create and moderate communities, and send private messages to other users.

Sends activities to Community: `Follow`, `Undo/Follow`, `Create`, `Update`, `Like`, `Dislike`, `Remove` (only admin/mod), `Delete` (only creator), `Undo` (only for own actions)

Receives activities from Community: `Accept/Follow`, `Announce`

Sends and receives activities from/to other users: `Create/Note`, `Update/Note`, `Delete/Note`, `Undo/Delete/Note` (all those related to private messages)

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/u/picard",
    "type": "Person",
    "name": "riker",
    "preferredUsername": "Jean-Luc Picard",
    "summary": "The user bio",
    "icon": {
        "type": "Image",
        "url": "https://enterprise.lemmy.ml/pictrs/image/DS3q0colRA.jpg"
    },
    "image": {
        "type": "Image",
        "url": "https://enterprise.lemmy.ml/pictrs/image/XenaYI5hTn.png"
    },
    "inbox": "https://enterprise.lemmy.ml/u/picard/inbox",
    "endpoints": {
        "sharedInbox": "https://enterprise.lemmy.ml/inbox"
    },
    "published": "2020-10-06T17:27:43.234391+00:00",
    "updated": "2020-10-08T11:27:17.905625+00:00",
    "publicKey": {
        "id": "https://enterprise.lemmy.ml/u/picard#main-key",
        "owner": "https://enterprise.lemmy.ml/u/picard",
        "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyH9iH83+idw/T4QpuRSY\n5YgQ/T5pJCNxvQWb6qcCu3gEVigfbreqZKJpOih4YT36wu4GjPfoIkbWJXcfcEzq\nMEQoYbPStuwnklpN2zj3lRIPfGLht9CAlENLWikTUoW5kZLyU6UQtOGdT2b1hDuK\nsUEn67In6qYx6pal8fUbO6X3O2BKzGeofnXgHCu7QNIuH4RPzkWsLhvwqEJYP0zG\nodao2j+qmhKFsI4oNOUCGkdJejO7q+9gdoNxAtNNKilIOwUFBYXeZJb+XGlzo0X+\n70jdJ/xQCPlPlItU4pD/0FwPLtuReoOpMzLi20oDsPXJBvn+/NJaxqDINuywcN5p\n4wIDAQAB\n-----END PUBLIC KEY-----\n"
    }
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `name` | yes | Name of the actor |
| `preferredUsername` | no | Displayname |
| `summary` | no | User bio |
| `icon` | no | The user's avatar, shown next to the username |
| `image` | no | The user's banner, shown on top of the profile |
| `inbox` | no | ActivityPub inbox URL |
| `endpoints` | no | Contains URL of shared inbox |
| `published` | no | Datetime when the user signed up |
| `updated` | no | Datetime when the user profile was last changed |
| `publicKey` | yes | The public key used to verify signatures from this actor |

## Objects

### Post

A page with title, and optional URL and text content. The URL often leads to an image, in which case a thumbnail is included. Each post belongs to exactly one community.

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://voyager.lemmy.ml/post/29",
    "type": "Page",
    "attributedTo": "https://voyager.lemmy.ml/u/picard",
    "to": "https://voyager.lemmy.ml/c/main",
    "summary": "Test thumbnail 2",
    "content": "blub blub",
    "url": "https://voyager.lemmy.ml:/pictrs/image/fzGwCsq7BJ.jpg",
    "image": {
        "type": "Image",
        "url": "https://voyager.lemmy.ml/pictrs/image/UejwBqrJM2.jpg"
    },
    "commentsEnabled": true,
    "sensitive": false,
    "stickied": false,
    "published": "2020-09-24T17:42:50.396237+00:00",
    "updated": "2020-09-24T18:31:14.158618+00:00"
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `attributedTo` | yes | ID of the user which created this post |
| `to` | yes | ID of the community where it was posted to |
| `summary` | yes | Title of the post |
| `content` | no | Body of the post |
| `url` | no | An arbitrary link to be shared |
| `image` | no | Thumbnail for `url`, only present if it is an image link |
| `commentsEnabled` | yes | False indicates that the post is locked, and no comments can be added |
| `sensitive` | yes | True marks the post as NSFW, blurs the thumbnail and hides it from users with NSFW settign disabled |
| `stickied` | yes | True means that it is shown on top of the community |
| `published` | no | Datetime when the post was created |
| `updated` | no | Datetime when the post was edited (not present if it was never edited) |

### Comment

A reply to a post, or reply to another comment. Contains only text (including references to other users or communities). Lemmy displays comments in a tree structure.

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/comment/95",
    "type": "Note",
    "attributedTo": "https://enterprise.lemmy.ml/u/picard",
    "to": "https://enterprise.lemmy.ml/c/main",
    "content": "mmmk",
    "inReplyTo": [
        "https://enterprise.lemmy.ml/post/38",
        "https://voyager.lemmy.ml/comment/73"
    ],
    "published": "2020-10-06T17:53:22.174836+00:00",
    "updated": "2020-10-06T17:53:22.174836+00:00"
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `attributedTo` | yes | ID of the user who created the comment |
| `to` | yes | Community where the comment was made |
| `content` | yes | The comment text |
| `inReplyTo` | yes | IDs of the post where this comment was made, and the parent comment. If this is a top-level comment, `inReplyTo` only contains the post |
| `published` | no | Datetime when the comment was created |
| `updated` | no | Datetime when the comment was edited (not present if it was never edited) |

### Private Message

A direct message from one user to another. Can not include additional users. Threading is not implemented yet, so the `inReplyTo` field is missing.

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/private_message/34",
    "type": "Note",
    "attributedTo": "https://enterprise.lemmy.ml/u/picard",
    "to": "https://voyager.lemmy.ml/u/janeway",
    "content": "test",
    "published": "2020-10-08T19:10:46.542820+00:00",
    "updated": "2020-10-08T20:13:52.547156+00:00"
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `attributedTo` | ID of the user who created this private message |
| `to` | ID of the recipient |
| `content` | yes | The text of the private message |
| `published` | no | Datetime when the message was created |
| `updated` | no | Datetime when the message was edited (not present if it was never edited) |

## Activities

### Follow

When the user clicks "Subscribe" in a community, a `Follow` is sent. The community automatically responds with an `Accept/Follow`.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/follow/2e4784b7-4edf-4fa1-a352-674d5d5f8891",
    "type": "Follow",
    "actor": "https://enterprise.lemmy.ml/u/picard",
    "to": "https://ds9.lemmy.ml/c/main",
    "object": "https://ds9.lemmy.ml/c/main"
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `actor` | yes | The user that is sending the follow request |
| `object` | yes | The community to be followed |

### Accept Follow

Automatically sent by the community in response to a `Follow`. At the same time, the community adds this user to its followers list.

Sent by: Community

Sent to: User

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://ds9.lemmy.ml/activities/accept/5314bf7c-dab8-4b01-baf2-9be11a6a812e",
    "type": "Accept",
    "actor": "https://ds9.lemmy.ml/c/main",
    "to": "https://enterprise.lemmy.ml/u/picard",
    "object": {
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": "https://enterprise.lemmy.ml/activities/follow/2e4784b7-4edf-4fa1-a352-674d5d5f8891",
        "type": "Follow",
        "object": "https://ds9.lemmy.ml/c/main",
        "actor": "https://enterprise.lemmy.ml/u/picard"
    }
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `actor` | yes | The same community as in the `Follow` activity |
| `to` | no | ID of the user which sent the `Follow` |
| `object` | yes | The previously sent `Follow` activity |

### Unfollow

Clicking on the unsubscribe button in a community causes an `Undo/Follow` to be sent. The community removes the user from its follower list after receiving it.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "http://lemmy-alpha:8541/activities/undo/2c624a77-a003-4ed7-91cb-d502eb01b8e8",
    "type": "Undo",
    "actor": "http://lemmy-alpha:8541/u/lemmy_alpha",
    "to": "http://lemmy-beta:8551/c/main",
    "object": {
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": "http://lemmy-alpha:8541/activities/follow/f0d732e7-b1e7-4857-a5e0-9dc83c3f7ee8",
        "type": "Follow",
        "actor": "http://lemmy-alpha:8541/u/lemmy_alpha",
        "object": "http://lemmy-beta:8551/c/main"
    }
}
```
### Create or Update Post

When a user creates a new post, it is sent to the respective community. Editing a previously created post sends an almost identical activity, except the `type` being `Update`. We don't support mentions in posts yet.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/create/6e11174f-501a-4531-ac03-818739bfd07d",
    "type": "Create",
    "actor": "https://enterprise.lemmy.ml/u/riker",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "cc": [
      "https://ds9.lemmy.ml/c/main/"
    ],
    "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `type` | yes | either `Create` or `Update` |
| `cc` | yes | Community where the post is being made |
| `object` | yes | The post being created |

### Create or Update Comment

A reply to a post, or to another comment. Can contain mentions of other users. Editing a previously created post sends an almost identical activity, except the `type` being `Update`.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/create/6f52d685-489d-4989-a988-4faedaed1a70",
    "type": "Create",
    "actor": "https://enterprise.lemmy.ml/u/riker",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "tag": [{
        "type": "Mention",
        "name": "@sisko@ds9.lemmy.ml",
        "href": "https://ds9.lemmy.ml/u/sisko"
    }],
    "cc": [
        "https://ds9.lemmy.ml/c/main/",
        "https://ds9.lemmy.ml/u/sisko"
    ],
    "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `tag` | no | List of users which are mentioned in the comment (like `@user@example.com`) |
| `cc` | yes | Community where the post is being made, as well as any mentioned users |
| `object` | yes | The comment being created |

### Like Post or Comment

An upvote for a post or comment.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/like/8f3f48dd-587d-4624-af3d-59605b7abad3",
    "type": "Like",
    "actor": "https://enterprise.lemmy.ml/u/riker",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "cc": [
        "https://ds9.lemmy.ml/c/main/"
    ],
    "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `cc` | yes | ID of the community where the post/comment is |
| `object` | yes | The post or comment being upvoted |

### Dislike Post or Comment

A downvote for a post or comment.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/dislike/fd2b8e1d-719d-4269-bf6b-2cadeebba849",
    "type": "Dislike",
    "actor": "https://enterprise.lemmy.ml/u/riker",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "cc": [
      "https://ds9.lemmy.ml/c/main/"
    ],
    "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `cc` | yes | ID of the community where the post/comment is |
| `object` | yes | The post or comment being upvoted |

### Delete Post or Comment

Deletes a previously created post or comment. This can only be done by the original creator of that post/comment.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/delete/f1b5d57c-80f8-4e03-a615-688d552e946c",
    "type": "Delete",
    "actor": "https://enterprise.lemmy.ml/u/riker",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "cc": [
        "https://enterprise.lemmy.ml/c/main/"
    ],
    "object": "https://enterprise.lemmy.ml/post/32"
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `cc` | yes | ID of the community where the post/comment is |
| `object` | yes | ID of the post or comment being deleted |

### Remove Post or Comment

Removes a post or comment. This can only be done by a community mod, or by an admin on the instance where the community is hosted.

Sent by: User

Sent to: Community

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://ds9.lemmy.ml/activities/remove/aab93b8e-3688-4ea3-8212-d00d29519218",
    "type": "Remove",
    "actor": "https://ds9.lemmy.ml/u/sisko",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "cc": [
        "https://ds9.lemmy.ml/c/main/"
    ],
    "object": "https://enterprise.lemmy.ml/comment/32"
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `cc` | yes | ID of the community where the post/comment is |
| `object` | yes | ID of the post or comment being removed |

### Undo

Reverts a previous activity, can only be done by the `actor` of `object`. In case of a `Like` or `Dislike`, the vote count is changed back. In case of a `Delete` or `Remove`, the post/comment is restored. The `object` is regenerated from scratch, as such the activity ID and other fields are different.

Sent by: User

Sent to: Community

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://ds9.lemmy.ml/activities/undo/70ca5fb2-e280-4fd0-a593-334b7f8a5916",
  "type": "Undo",
  "actor": "https://ds9.lemmy.ml/u/sisko",
  "to": "https://www.w3.org/ns/activitystreams#Public",
  "cc": [
    "https://ds9.lemmy.ml/c/main/"
  ],
  "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `object` | yes | Any `Like`, `Dislike`, `Delete` or `Remove` activity as described above |

### Announce

When the community receives a post or comment activity, it wraps that into an `Announce` and sends it to all followers.

Sent by: Community

Sent to: User

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://ds9.lemmy.ml/activities/announce/b98382e8-6cb1-469e-aa1f-65c5d2c31cc4",
  "type": "Announce",
  "actor": "https://ds9.lemmy.ml/c/main",
  "to": "https://www.w3.org/ns/activitystreams#Public",
  "cc": [
    "https://ds9.lemmy.ml/c/main/followers"
  ],
  "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `object` | yes | Any `Create`, `Update`, `Like`, `Dislike`, `Delete` `Remove` or `Undo` activity as described above |

### Create or Update Private message 

Creates a new private message between two users.

Sent by: User

Sent to: User

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://ds9.lemmy.ml/activities/create/202daf0a-1489-45df-8d2e-c8a3173fed36",
    "type": "Create",
    "actor": "https://ds9.lemmy.ml/u/sisko",
    "to": "https://enterprise.lemmy.ml/u/riker/inbox",
    "object": ...
}
```

| Field Name | Mandatory | Description |
|---|---|---|
| `type` | yes | Either `Create` or `Update` |

### Delete Private Message

Deletes a previous private message.

Sent by: User

Sent to: User

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://ds9.lemmy.ml/activities/delete/2de5a5f3-bf26-4949-a7f5-bf52edfca909",
    "type": "Delete",
    "actor": "https://ds9.lemmy.ml/u/sisko",
    "to": "https://enterprise.lemmy.ml/u/riker/inbox",
    "object": "https://ds9.lemmy.ml/private_message/341"
}
```

### Undo Delete Private Message

Restores a previously deleted private message. The `object` is regenerated from scratch, as such the activity ID and other fields are different.

Sent by: User

Sent to: User

```json
{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://ds9.lemmy.ml/activities/undo/b24bc56d-5db1-41dd-be06-3f1db8757842",
    "type": "Undo",
    "actor": "https://ds9.lemmy.ml/u/sisko",
    "to": "https://enterprise.lemmy.ml/u/riker/inbox",
    "object": ...
}
```