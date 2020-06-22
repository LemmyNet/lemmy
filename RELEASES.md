# Lemmy v0.7.0 Release (2020-06-2X)

## Breaking Change to our image server: Pictshare to Pict-rs migration guide

This release replaces [pictshare](https://github.com/HaschekSolutions/pictshare) with [pict-rs](https://git.asonix.dog/asonix/pict-rs), and a script must be run on your server to upgrade.

To update, run:

```
cd /lemmy
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/migrate-pictshare-to-pictrs.bash
sudo bash migrate-pictshare-to-pictrs.bash
```

You'll also have to update your nginx config, use the [one here](https://github.com/LemmyNet/lemmy/blob/master/ansible/templates/nginx.conf).

*You'll have to log in again to pick up your avatar*

Apart from that, we've closed [~90 issues!](https://github.com/LemmyNet/lemmy/milestone/16?closed=1), including:

- Site-wide list of recent comments.
- Reconnecting websockets.
- Lots more themes, including a default light one.
- Expandable embeds for post links (and thumbnails), from iframely.
- Better icons.
- Emoji autocomplete to post and message bodies, and an Emoji Picker.
- Post body now searchable.
- Community title and description is now searchable.
- Simplified cross-posts.
- Better documentation.
- LOTS more languages.
- Lots of bugs squashed.

# Lemmy v0.6.0 Release (2020-01-16)

`v0.6.0` is here, and we've closed [41 issues!](https://github.com/LemmyNet/lemmy/milestone/15?closed=1) 

This is the biggest release by far:

- Avatars!
- Optional Email notifications for username mentions, post and comment replies.
- Ability to change your password and email address.
- Can set a custom language.
- Lemmy-wide settings to disable downvotes, and close registration.
- A better documentation system, hosted in lemmy itself.
- [Huge DB performance gains](https://github.com/LemmyNet/lemmy/issues/411) (everthing down to < `30ms`) by using materialized views. 
- Fixed major issue with similar post URL and title searching.
- Upgraded to Actix `2.0`
- Faster comment / post voting.
- Better small screen support.
- Lots of bug fixes, refactoring of back end code.

Another major announcement is that Lemmy now has another lead developer besides me, [@felix@radical.town](https://radical.town/@felix). Theyve created a better documentation system, implemented RSS feeds, simplified docker and project configs, upgraded actix, working on federation, a whole lot else.

https://dev.lemmy.ml
