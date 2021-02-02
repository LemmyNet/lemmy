# Lemmy v0.9.4 Pre-Release (2021-02-02)

## Changes

### Lemmy

- Fixed a critical bug with votes and comment unlike responses not being `0` for your user.
- Fixed a critical bug with comment creation not checking if its parent comment is in the post.
- Serving proper activities for community outbox.
- Added some active user counts, including `users_active_day`, `users_active_week`, `users_active_month`, `users_active_half_year` to `SiteAggregates` and `CommunityAggregates`. (Also added to lemmy-ui)
- Made sure banned users can't follow.
- Added `FederatedInstances` to `SiteResponse`, to show allowed and blocked instances. (Also added to lemmy-ui)
- Added a `MostComments` sort for posts. (Also added to lemmy-ui)

### Lemmy-UI

- Added a scroll position restore to lemmy-ui.
- Reworked the combined inbox so incoming comments don't wipe out your current form.
- Fixed an updated bug on the user page.
- Fixed cross-post titles and body getting clipped.
- Fixing the post creation title height.
- Squashed some other smaller bugs.

# Lemmy v0.9.0 Release (2021-01-25)

## Changes

Since our last release in October of last year, and we've had [~450](https://github.com/LemmyNet/lemmy/compare/v0.8.0...main) commits.

The biggest changes, as we'll outline below, are a re-work of Lemmy's database structure, a `v2` of Lemmy's API, and activitypub compliance fixes. The new re-worked DB is much faster, easier to maintain, and [now supports hierarchical rather than flat objects in the new API](https://github.com/LemmyNet/lemmy/issues/1275).

We've also seen the first release of [Lemmur](https://github.com/krawieck/lemmur/releases/tag/v0.1.1), an android / iOS (soon) / windows / linux client, as well as [Lemmer](https://github.com/uuttff8/Lemmy-iOS), a native iOS client. Much thanks to @krawieck, @shilangyu, and @uuttff8 for making these great clients. If you can, please contribute to their [patreon](https://www.patreon.com/lemmur) to help fund lemmur development.

## LemmyNet projects

### Lemmy Server

- [Moved views from SQL to Diesel](https://github.com/LemmyNet/lemmy/issues/1275). This was a spinal replacement for much of lemmy.
  - Removed all the old fast_tables and triggers, and created new aggregates tables.
- Added a `v2` of the API to support the hierarchical objects created from the above changes.
- Moved continuous integration to [drone](https://cloud.drone.io/LemmyNet/lemmy/), now includes formatting, clippy, and cargo build checks, unit testing, and federation testing. [Drone also deploys both amd64 and arm64 images to dockerhub.](https://hub.docker.com/r/dessalines/lemmy)
- Split out documentation into git submodule.
- Shortened slur filter to avoid false positives.
- Added query performance testing and comparisons. Added indexes to make sure every query is `< 30 ms`.
- Added compilation time testing.

### Federation

This release includes some bug fixes for federation, and some changes to get us closer to compliance with the ActivityPub standard.

- [Community bans now federating](https://github.com/LemmyNet/lemmy/issues/1287).
- [Local posts sometimes got marked as remote](https://github.com/LemmyNet/lemmy/issues/1302).
- [Creator of post/comment was not notified about new child comments](https://github.com/LemmyNet/lemmy/issues/1325).
- [Community deletion now federated](https://github.com/LemmyNet/lemmy/issues/1256).

None of these are breaking changes, so federation between 0.9.0 and 0.8.11 will work without problems.

### Lemmy javascript / typescript client

- Updated the [lemmy-js-client](https://github.com/LemmyNet/lemmy-js-client) to use the new `v2` API. Our API docs now reference this project's files, to show what the http / websocket forms and responses should look like.
- Drone now handles publishing its [npm packages.](https://www.npmjs.com/package/lemmy-js-client)

### Lemmy-UI

- Updated it to use the `v2` API via `lemmy-js-client`, required changing nearly every component.
- Added a live comment count.
- Added drone deploying, and builds for ARM.
- Fixed community link wrapping.
- Various other bug fixes.


### Lemmy Docs

- We moved documentation into a separate git repository, and support translation for the docs now!
- Moved our code of conduct into the documentation.

## Upgrading

If you'd like to make a DB backup before upgrading, follow [this guide](https://lemmy.ml/docs/en/administration/backup_and_restore.html).

- [Upgrade with manual Docker installation](https://lemmy.ml/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://lemmy.ml/docs/en/administration/install_ansible.html)

# Lemmy v0.8.0 Release (2020-10-16)

## Changes

We've been working at warp speed since our `v0.7.0` release in June, adding over [870 commits](https://github.com/LemmyNet/lemmy/compare/v0.7.0...main) since then. :sweat: 

Here are some of the bigger changes:

### LemmyNet projects

- Created [LemmyNet](https://github.com/LemmyNet), where all lemmy-related projects live.
- Split out the frontend into a separete repository, [lemmy-ui](https://github.com/LemmyNet/lemmy-ui)
- Created a [lemmy-js-client](https://github.com/LemmyNet/lemmy-js-client), for any js / typescript developers.
- Split out i18n [lemmy-translations](https://github.com/LemmyNet/lemmy-translations), that any app or site developers can import and use. Lemmy currently supports [~30 languages!](https://weblate.yerbamate.ml/projects/lemmy/lemmy/)

### Lemmy Server

#### Federation

- The first **federation public beta release**, woohoo :fireworks: 
- All Lemmy functionality now works over ActivityPub (except turning remote users into mods/admins)
- Instance allowlist and blocklist
- Documentation for [admins](https://lemmy.ml/docs/administration_federation.html) and [devs](https://lemmy.ml/docs/contributing_federation_overview.html) on how federation works
- Upgraded to newest versions of @asonix activitypub libraries
- Full local federation setup for manual testing
- Automated testing for nearly every federation action
- Many additional security checks
- Lots and lots of refactoring
- Asynchronous sending of outgoing activities

### User Interface

- Separated the UI from the server code, in [lemmy-ui](https://github.com/LemmyNet/lemmy-ui).
- The UI can now read with javascript disabled! 
- It's now a fully isomorphic application using [inferno-isomorphic](https://infernojs.org/docs/guides/isomorphic). This means that page loads are now much faster, as the server does the work.
- The UI now also supports open-graph and twitter cards! Linking to lemmy posts (from whatever platform you use) looks pretty now: ![](https://i.imgur.com/6TZ2v7s.png)
- Improved the search page ( more features incoming ).
- The default view is now `Local`, instead of `All`, since all would show all federated posts.
- User settings are now shared across browsers ( a page refresh will pick up changes ).
- A much leaner mobile view.

#### Backend

- Re-organized the rust codebase into separate workspaces for backend and frontend.
- Removed materialized views, making the database **a lot faster**.
- New post sorts `Active` (previously called hot), and `Hot`. Active shows posts with recent comments, hot shows highly ranked posts.
- New sort for `Local` ( meaning from local communities).
- Customizeable site, user, and community icons and banners.
- Added user preferred names / display names, bios, and cakedays.
- Visual / Audio captchas through the lemmy API.
- Lots of API field verifications.
- Upgraded to pictrs-v2 ( thanks to @asonix )
- Wayyy too many bugfixes to count.

## Contributors

We'd also like to thank both the [NLnet foundation](https://nlnet.nl/) for their support in allowing us to work full-time on Lemmy ( as well as their support for [other important open-source projects](https://nlnet.nl/project/current.html) ), [those who sponsor us](https://lemmy.ml/sponsors), and those who [help translate Lemmy](https://weblate.yerbamate.ml/projects/lemmy/). Every little bit does help. We remain committed to never allowing advertisements, monetizing, or venture-capital in Lemmy; software should be communal, and should benefit humanity, not a small group of company owners. 

## Upgrading

- [with manual Docker installation](https://lemmy.ml/docs/administration_install_docker.html#updating)
- [with Ansible installation](https://lemmy.ml/docs/administration_install_ansible.html)

## Testing Federation

Federation is finally ready in Lemmy, pending possible bugs or other issues. So for now we suggest to enable federation only on test servers, or try it on our own test servers ( [enterprise](https://enterprise.lemmy.ml/), [ds9](https://ds9.lemmy.ml/), [voyager](https://voyager.lemmy.ml/) ).

If everything goes well, after a few weeks we will enable federation on lemmy.ml, at first with a limited number of trusted instances. We will also likely change the domain to https://lemmy.ml . Keep in mind that changing domains after turning on federation will break things.

To enable on your instance, edit your [lemmy.hjson](https://github.com/LemmyNet/lemmy/blob/main/config/defaults.hjson#L60) federation section to `enabled: true`, and restart.

### Connecting to another server

The server https://ds9.lemmy.ml has open federation, so after either adding it to the `allowed_instances` list in your `config.hjson`, or if you have open federation, you don't need to add it explicitly.

To federate / connect with a server, type in `!community_name@server.tld`, in your server's search box [like so](https://voyager.lemmy.ml/search/q/!main%40ds9.lemmy.ml/type/All/sort/TopAll/page/1).

To connect with the `main` community on ds9, the search is `!main@ds9.lemmy.ml`.

You can then click the community, and you will see a local version of the community, which you can subscribe to. New posts and comments from `!main@ds9.lemmy.ml` will now show up on your front page, or `/c/All`

# Lemmy v0.7.40 Pre-Release (2020-08-05)

We've [added a lot](https://github.com/LemmyNet/lemmy/compare/v0.7.40...v0.7.0) in this pre-release:

- New post sorts `Active` (previously called hot), and `Hot`. Active shows posts with recent comments, hot shows highly ranked posts.
- Customizeable site icon and banner, user icon and banner, and community icon and banner.
- Added user preferred names / display names, bios, and cakedays.
- User settings are now shared across browsers (a page refresh will pick up changes).
- Visual / Audio captchas through the lemmy API.
- Lots of UI prettiness.
- Lots of bug fixes.
- Lots of additional translations.
- Lots of federation prepping / additions / refactors.

This release removes the need for you to have a pictrs nginx route (the requests are now routed through lemmy directly). Follow the upgrade instructions below to replace your nginx with the new one.

## Upgrading

**With Ansible:**

```
# run these commands locally
git pull
cd ansible
ansible-playbook lemmy.yml
```

**With manual Docker installation:**
```
# run these commands on your server
cd /lemmy
wget https://raw.githubusercontent.com/LemmyNet/lemmy/master/ansible/templates/nginx.conf
# Replace the {{ vars }}
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
sudo nginx -s reload
wget https://raw.githubusercontent.com/LemmyNet/lemmy/master/docker/prod/docker-compose.yml
sudo docker-compose up -d
```


# Lemmy v0.7.0 Release (2020-06-23)

This release replaces [pictshare](https://github.com/HaschekSolutions/pictshare)
with [pict-rs](https://git.asonix.dog/asonix/pict-rs), which improves performance
and security.

Overall, since our last major release in January (v0.6.0), we have closed over
[100 issues!](https://github.com/LemmyNet/lemmy/milestone/16?closed=1)

- Site-wide list of recent comments
- Reconnecting websockets
- Many more themes, including a default light one.
- Expandable embeds for post links (and thumbnails), from
[iframely](https://github.com/itteco/iframely)
- Better icons
- Emoji autocomplete to post and message bodies, and an Emoji Picker
- Post body now searchable
- Community title and description is now searchable
- Simplified cross-posts
- Better documentation
- LOTS more languages
- Lots of bugs squashed
- And more ...

## Upgrading

Before starting the upgrade, make sure that you have a working backup of your
database and image files. See our
[documentation](https://lemmy.ml/docs/administration_backup_and_restore.html)
for backup instructions.

**With Ansible:**

```
# deploy with ansible from your local lemmy git repo
git pull
cd ansible
ansible-playbook lemmy.yml
# connect via ssh to run the migration script
ssh your-server
cd /lemmy/
wget https://raw.githubusercontent.com/LemmyNet/lemmy/master/docker/prod/migrate-pictshare-to-pictrs.bash
chmod +x migrate-pictshare-to-pictrs.bash
sudo ./migrate-pictshare-to-pictrs.bash
```

**With manual Docker installation:**
```
# run these commands on your server
cd /lemmy
wget https://raw.githubusercontent.com/LemmyNet/lemmy/master/ansible/templates/nginx.conf
# Replace the {{ vars }}
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
sudo nginx -s reload
wget https://raw.githubusercontent.com/LemmyNet/lemmy/master/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/LemmyNet/lemmy/master/docker/prod/migrate-pictshare-to-pictrs.bash
chmod +x migrate-pictshare-to-pictrs.bash
sudo bash migrate-pictshare-to-pictrs.bash
```

**Note:** After upgrading, all users need to reload the page, then logout and
login again, so that images are loaded correctly.

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

https://lemmy.ml
