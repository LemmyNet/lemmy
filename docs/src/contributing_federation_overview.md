# Federation


This document is for anyone who wants to know how Lemmy federation works, without being overly technical. It is meant provide a high-level overview of ActivityPub federation in Lemmy. If you are implementing ActivityPub yourself and want to be compatible with Lemmy, read our [ActivityPub API outline](contributing_apub_api_outline.md).

## Documentation conventions

To keep things simple, sometimes you will see things formatted like `Create/Note` or `Delete/Event` or `Undo/Follow`. The thing before the slash is the Activity, and the thing after the slash is the Object inside the Activity, in an `object` property. So these are to be read as follows:

* `Create/Note`: a `Create` activity containing a `Note` in the `object` field 
* `Delete/Event`: a `Delete` activity containing an `Event` in the `object` field
* `Undo/Follow`: an `Undo` activity containing a `Follow` in the `object` field

In Lemmy we use some specific terms to refer to ActivityPub items. They are essentially our specific implementations of well-known ActivityPub concepts:

- Community: `Group`
- User: `Person`
- Post: `Page`
- Comment: `Note`

This document has three main sections:

* __Federation philosophy__ lays out the general model of how this is intended to federate
* __User Activities__ describes which actions that a User can take to interact
* __Community Activities__ describes what the Community does in response to certain User actions

## Federation philosophy

The primary Actor in Lemmy is the Community. Each community resides on a single instance, and consists of a list of Posts and a list of followers. The primary interaction is that of a User sending a Post or Comment related activity to the Community inbox, which then announces it to all its followers. 

Each Community has a specific creator User, who is responsible for setting rules, appointing moderators, and removing content that violates the rules.

Besides moderation on the community level, each instance has a set of administrator Users, who have the power to do site-wide removals and bans.

Users follow Communities that they are interested in, in order to receive Posts and Comments. They also vote on Posts and Comments, as well as creating new ones. Comments are organised in a tree structure and commonly sorted by number of votes. Direct messages between Users are also supported.

Users can not follow each other, and neither can Communities follow anything.

Our federation implementation is already feature complete, but so far we haven't focused at all on complying with the ActivityPub spec. As such, Lemmy is likely not compatible with implementations which expect to send and receive valid activities. This is something we plan to fix in the near future. Check out [#698](https://github.com/LemmyNet/lemmy/issues/698) for an overview of our deviations.

## User Activities

### Follow a Community

Each Community page has a "Follow" button. Clicking this triggers a `Follow` activity to be sent from the user to the Community inbox. The Community will automatically respond with an `Accept/Follow` activity to the user inbox. It will also add the user to its list of followers, and deliver any activities about Posts/Comments in the Community to the user.

### Unfollow a Community

After following a Community, the "Follow" button is replaced by "Unfollow". Clicking this sends an `Undo/Follow` activity to the Community inbox. The Community removes the User from its followers list and doesn't send any activities to it anymore.

### Create a Post

When a user creates a new Post in a given Community, it is sent as `Create/Page` to the  Community
inbox. 

### Create a Comment

When a new Comment is created for a Post, both the Post ID and the parent Comment ID (if it exists)
are written to the `in_reply_to` field. This allows assigning it to the correct Post, and building
the Comment tree. It is then sent to the Community inbox as `Create/Note`

The origin instance also scans the Comment for any User mentions, and sends the `Create/Note` to
those Users as well.

### Edit a Post

Changes the content of an existing Post. Can only be done by the creating User.

### Edit a Comment

Changes the content of an existing Comment. Can only be done by the creating User.

### Likes and Dislikes

Users can like or dislike any Post or Comment. These are sent as `Like/Page`, `Dislike/Note` etc to the Community inbox.

### Deletions

The creator of a Post, Comment or Community can delete it. It is then sent to the Community followers. The item is then hidden from all users.

### Removals

Mods can remove Posts and Comments from their Communities. Admins can remove any Posts or Comments on the entire site. Communities can also be removed by admins. The item is then hidden from all users.

Removals are sent to all followers of the Community, so that they also take effect there. The exception is if an admin removes an item from a Community which is hosted on a different instance. In this case, the removal only takes effect locally.

### Revert a previous Action

We don't delete anything from our database, just hide it from users. Deleted or removed Communities/Posts/Comments have a "restore" button. This button generates an `Undo` activity which sets the original delete/remove activity as object, such as `Undo/Remove/Post` or `Undo/Delete/Community`.

Clicking on the upvote button of an already upvoted post/comment (or the downvote button of an already downvoted post/comment) also generates an `Undo`. In this case and `Undo/Like/Post` or `Undo/Dislike/Comment`.

### Create private message

User profiles have a "Send Message" button, which opens a dialog permitting to send a private message to this user. It is sent as a `Create/Note` to the user inbox. Private messages can only be directed at a single User.

### Edit private message

`Update/Note` changes the text of a previously sent message

### Delete private message

`Delete/Note` deletes a private message.

### Restore private message

`Undo/Delete/Note` reverts the deletion of a private message.

## Community Activities

The Community is essentially a bot, which will only do anything in reaction to actions from Users. The User who first created the Community becomes the first moderator, and can add additional moderators. In general, whenever the Community receives a valid activity in its inbox, that activity is forwarded to all its followers.

### Accept follow

If the Community receives a `Follow` activity, it automatically responds with `Accept/Follow`. It also adds the User to its list of followers. 

### Unfollow

Upon receiving an `Undo/Follow`, the Community removes the User from its followers list.
 
### Announce

If the Community receives any Post or Comment related activity (Create, Update, Like, Dislike, Remove, Delete, Undo), it will Announce this to its followers. For this, an Announce is created with the Community as actor, and the received activity as object. Following instances thus stay updated about any actions in Communities they follow.

### Delete Community

If the creator or an admin deletes the Community, it sends a `Delete/Group` to all its followers.
