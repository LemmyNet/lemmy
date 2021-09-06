# Lemmy v0.12.2 Release (2021-09-06)

- Fixing ARM64 builds.
- Fixing missing Arabic language in UI.
- Fixing wrongly shown subscribed communities on other user's profiles. [#402](https://github.com/LemmyNet/lemmy-ui/issues/402)
- Adding a robots.txt, thanks to @mahanstreamer. [#401](https://github.com/LemmyNet/lemmy-ui/pull/401)

# Lemmy v0.12.1 Release (2021-09-04)

Fixed several critical websocket bugs.

- Wasn't correctly getting comment parent user for mark as read. Fixes [#1767](https://github.com/LemmyNet/lemmy/issues/1767)
- Was using all recipients for simple comment return. Fixes [#1766](https://github.com/LemmyNet/lemmy/issues/1766)
- Fix comment scrolling bug. Fixes [#394](https://github.com/LemmyNet/lemmy-ui/issues/394)

# Lemmy v0.12.0 Release (2021-09-03)

## Changes

Since our last release in April, we've had [~80](https://github.com/LemmyNet/lemmy/compare/0.11.0...main) commits to Lemmy.

### Lemmy Server

#### Major Changes

*Note: Issue links are below.*

- @nutomic did a major rewrite of the federation code. It is much simpler now, and reduced from 8000 lines of code to 6400. Functionality is mostly the same, but future changes will be much easier.
- You can now block users and communities, and their posts / comments won't show up in your feed.
- Removed IFramely: Lemmy can now fetch site metadata on its own.
- New API docs at: https://join-lemmy.org/api

#### General

- Fix prod deploy script and clippy ([#1724](https://github.com/LemmyNet/lemmy/issues/1724))
- Fix image uploads. Fixes [#1725](https://github.com/LemmyNet/lemmy/issues/1725) ([#1734](https://github.com/LemmyNet/lemmy/issues/1734))
- Adding more site setup vars. Fixes [#678](https://github.com/LemmyNet/lemmy/issues/678) ([#1718](https://github.com/LemmyNet/lemmy/issues/1718))
- Dont append ? to url when cleaning it ([#1716](https://github.com/LemmyNet/lemmy/issues/1716))
- User / community blocking. Fixes [#426](https://github.com/LemmyNet/lemmy/issues/426) ([#1604](https://github.com/LemmyNet/lemmy/issues/1604))
- Swap out iframely ([#1706](https://github.com/LemmyNet/lemmy/issues/1706))
- Adding ModTransferCommunity to modlog in API. Fixes [#1437](https://github.com/LemmyNet/lemmy/issues/1437)
- Make sure bots aren't included in aggregate counts ([#1705](https://github.com/LemmyNet/lemmy/issues/1705))
- Don't allow deleted users to do actions. Fixes [#1656](https://github.com/LemmyNet/lemmy/issues/1656) ([#1704](https://github.com/LemmyNet/lemmy/issues/1704))
- When banning a user, remove communities they've created ([#1700](https://github.com/LemmyNet/lemmy/issues/1700))
- Distribute Lemmy via crates.io
- Simplify config using macros ([#1686](https://github.com/LemmyNet/lemmy/issues/1686))
- Simplify lemmy_context() function (dont return errors)
- Blank out extra info for deleted or removed content. Fixes [#1679](https://github.com/LemmyNet/lemmy/issues/1679) ([#1680](https://github.com/LemmyNet/lemmy/issues/1680))
- Add show_new_posts_notifs setting. Fixes [#1664](https://github.com/LemmyNet/lemmy/issues/1664) ([#1665](https://github.com/LemmyNet/lemmy/issues/1665))
- Adding shortname fetching for users and communities. Fixes [#1662](https://github.com/LemmyNet/lemmy/issues/1662) ([#1663](https://github.com/LemmyNet/lemmy/issues/1663))
- Upgrading deps, running clippy fix on nightly 1.55.0 ([#1638](https://github.com/LemmyNet/lemmy/issues/1638))
- Running clippy --fix ([#1647](https://github.com/LemmyNet/lemmy/issues/1647))
- Make captcha case-insensitive
- Remove tracking params from post url (fixes [#768](https://github.com/LemmyNet/lemmy/issues/768))
- Fix IPv6 port setup for Nginx ([#1636](https://github.com/LemmyNet/lemmy/issues/1636))
- Fix --cert-name for certbot. ([#1631](https://github.com/LemmyNet/lemmy/issues/1631))
- Change join.lemmy.ml to join-lemmy.org ([#1628](https://github.com/LemmyNet/lemmy/issues/1628))
- Upgrade pictrs. Fixes [#1599](https://github.com/LemmyNet/lemmy/issues/1599) ([#1600](https://github.com/LemmyNet/lemmy/issues/1600))
- Invalidate current logins on account deletion. Fixes [#1602](https://github.com/LemmyNet/lemmy/issues/1602) ([#1603](https://github.com/LemmyNet/lemmy/issues/1603))
- Upgrading api test deps ([#1608](https://github.com/LemmyNet/lemmy/issues/1608))
- Fix nsfw posts showing for non-logged in users. Fixes [#1614](https://github.com/LemmyNet/lemmy/issues/1614) ([#1615](https://github.com/LemmyNet/lemmy/issues/1615))
- Add additional slurs configuration option. Closes [#1464](https://github.com/LemmyNet/lemmy/issues/1464). ([#1612](https://github.com/LemmyNet/lemmy/issues/1612))
- Updating to rust 1.51.0 ([#1598](https://github.com/LemmyNet/lemmy/issues/1598))
- Remove brotli, zstd dependencies for faster compilation

#### API

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.11.0 -> 0.12.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.11.0...0.12.0-rc.1) .

#### Federation

- Move resolving of activitypub objects to separate api endpoint (fixes #1584)
- Rewrite remaining activities ([#1712](https://github.com/LemmyNet/lemmy/issues/1712))
- Migrate comment inReplyTo field to single value (ref [#1454](https://github.com/LemmyNet/lemmy/issues/1454))
- Fix issue with protocol string in actor id generation ([#1668](https://github.com/LemmyNet/lemmy/issues/1668))


### Lemmy UI

- Integrating resolve_user into search. ([#377](https://github.com/LemmyNet/lemmy-ui/issues/377))
- Add lazy loading of images. Fixes [#329](https://github.com/LemmyNet/lemmy-ui/issues/329) ([#379](https://github.com/LemmyNet/lemmy-ui/issues/379))
- Adding vi, sk, mnc, and cy languages. ([#378](https://github.com/LemmyNet/lemmy-ui/issues/378))
- Feature/user community block ([#362](https://github.com/LemmyNet/lemmy-ui/issues/362))
- Swapping out iframely. ([#374](https://github.com/LemmyNet/lemmy-ui/issues/374))
- Adding mod transfer community ([#373](https://github.com/LemmyNet/lemmy-ui/issues/373))
- Remove content more ([#372](https://github.com/LemmyNet/lemmy-ui/issues/372))
- Scroll to comments on post's x comments button ([#312](https://github.com/LemmyNet/lemmy-ui/issues/312))
- Remove websocket connection messages. Fixes [#355](https://github.com/LemmyNet/lemmy-ui/issues/355)
- Center spinner, make smaller. Fixes [#203](https://github.com/LemmyNet/lemmy-ui/issues/203)
- Fix font issues. Fixes [#354](https://github.com/LemmyNet/lemmy-ui/issues/354)
- Have setting to disable notifs for new posts. Fixes [#132](https://github.com/LemmyNet/lemmy-ui/issues/132) ([#345](https://github.com/LemmyNet/lemmy-ui/issues/345))
- Remove max length constraints on actors. Fixes [#350](https://github.com/LemmyNet/lemmy-ui/issues/350) ([#351](https://github.com/LemmyNet/lemmy-ui/issues/351))
- Fix captcha replay bug. Fixes [#348](https://github.com/LemmyNet/lemmy-ui/issues/348) ([#349](https://github.com/LemmyNet/lemmy-ui/issues/349))
- Removing community and user routes in favor of shortnames. Fixes [#317](https://github.com/LemmyNet/lemmy-ui/issues/317) ([#343](https://github.com/LemmyNet/lemmy-ui/issues/343))
- Don't use default subscribed for communities page.
- Adding Listing type to communities page, default local. [#190](https://github.com/LemmyNet/lemmy-ui/issues/190) ([#342](https://github.com/LemmyNet/lemmy-ui/issues/342))
- Fix language bug on mobile browsers
- Collapse sidebar on mobile. Fixes [#335](https://github.com/LemmyNet/lemmy-ui/issues/335) ([#340](https://github.com/LemmyNet/lemmy-ui/issues/340))
- Re-organized components folder. ([#339](https://github.com/LemmyNet/lemmy-ui/issues/339))
- Fixing too many large spinners ([#337](https://github.com/LemmyNet/lemmy-ui/issues/337))
- Moving comment link to top bar. Fixes #307 ([#336](https://github.com/LemmyNet/lemmy-ui/issues/336))
- Fix/ws error messages ([#334](https://github.com/LemmyNet/lemmy-ui/issues/334))
- Make spinner bigger. Fixes [#203](https://github.com/LemmyNet/lemmy-ui/issues/203)
- Fix preview description html. Fixes [#110](https://github.com/LemmyNet/lemmy-ui/issues/110)
- Always show previous paginator, extract paginator component.
- Use better comment collapse icon, and add text. Fixes [#318](https://github.com/LemmyNet/lemmy-ui/issues/318)
- Fix symbols issue. Fixes [#319](https://github.com/LemmyNet/lemmy-ui/issues/319)
- Don't restore scroll position on page refresh. Fixes [#186](https://github.com/LemmyNet/lemmy-ui/issues/186)
- Insert triple backticks for 'code' button when multiple lines are selected. ([#311](https://github.com/LemmyNet/lemmy-ui/issues/311))
- Change join.lemmy-ui.ml to join-lemmy-ui.org
- Adding a comment here placeholder. Fixes [#301](https://github.com/LemmyNet/lemmy-ui/issues/301)
- Fix non-local community and person links. Fixes [#290](https://github.com/LemmyNet/lemmy-ui/issues/290)
- Fix navbar bug. Fixes [#289](https://github.com/LemmyNet/lemmy-ui/issues/289)
- Hide names of mods / admins without priveleges. Fixes [#285](https://github.com/LemmyNet/lemmy-ui/issues/285)
- Adding URL search type. Fixes [#286](https://github.com/LemmyNet/lemmy-ui/issues/286)
- Add a link to joinlemmy-ui on lemmy-ui.ml signup. Fixes [#235](https://github.com/LemmyNet/lemmy-ui/issues/235)
- Fix duped site description. Fixes [#281](https://github.com/LemmyNet/lemmy-ui/issues/281)

## Upgrade notes

### Servers

You may need to add this to your `lemmy.hjson`:

`pictrs_url: "http://pictrs:8080"`

If you'd like to make a DB backup before upgrading, follow [this guide](https://join.lemmy.ml/docs/en/administration/backup_and_restore.html).

To upgrade your instance to `v0.12.0`, simply follow the instructions in the documentation:

- [Upgrade with manual Docker installation](https://join.lemmy.ml/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://join.lemmy.ml/docs/en/administration/install_ansible.html)

### Clients / Apps

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.11.0 -> 0.12.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.11.0...0.12.0-rc.1) .

# Lemmy v0.11.3 Release (2021-07-30)

## Changes

Since our last release, we've had [~30](https://github.com/LemmyNet/lemmy/compare/0.11.0...main) commits to Lemmy, and [~60](https://github.com/LemmyNet/lemmy-ui/compare/0.11.0...main) to Lemmy UI.

### Lemmy Server

- Blank out extra info for deleted or removed content. Fixes [#1679](https://github.com/LemmyNet/Lemmy/issues/1679) 
- Add show_new_posts_notifs setting. Fixes [#1664](https://github.com/LemmyNet/Lemmy/issues/1664)
- Fix issue with protocol string in actor id generation [#1668](https://github.com/LemmyNet/Lemmy/issues/1668)
- Adding shortname fetching for users and communities. Fixes [#1662](https://github.com/LemmyNet/Lemmy/issues/1662)
- Make captcha case-insensitive.
- Remove tracking params from post url (fixes [#768](https://github.com/LemmyNet/Lemmy/issues/768))
- Upgrade pictrs. Fixes [#1599](https://github.com/LemmyNet/Lemmy/issues/1599)
- Invalidate current logins on account deletion. Fixes [#1602](https://github.com/LemmyNet/Lemmy/issues/1602)
- Fix nsfw posts showing for non-logged in users. Fixes [#1614](https://github.com/LemmyNet/Lemmy/issues/1614)
- Add additional slurs configuration option. Closes [#1464](https://github.com/LemmyNet/Lemmy/issues/1464).
- Updating to rust 1.51.0

### Lemmy UI

- Have setting to disable notifs for new posts. Fixes [#132](https://github.com/LemmyNet/lemmy-ui/issues/132)
- Remove max length constraints on actors. Fixes [#350](https://github.com/LemmyNet/lemmy-ui/issues/350)
- Fix captcha replay bug. Fixes [#348](https://github.com/LemmyNet/lemmy-ui/issues/348)
- Removing community and user routes in favor of shortnames. Fixes [#317](https://github.com/LemmyNet/lemmy-ui/issues/317)
- Add front end helpers 1 [(#346](https://github.com/LemmyNet/lemmy-ui/issues/346))
- Don't use default subscribed for communities page.
- Adding Listing type to communities page, default local. [#190](https://github.com/LemmyNet/lemmy-ui/issues/190)
- Fix language bug on mobile browsers.
- Collapse sidebar on mobile. Fixes [#335](https://github.com/LemmyNet/lemmy-ui/issues/335)
- Re-organized components folder. [(#339](https://github.com/LemmyNet/lemmy-ui/issues/339))
- Moving comment link to top bar. Fixes [#307](https://github.com/LemmyNet/lemmy-ui/issues/307)
- Make spinner bigger. Fixes [#203](https://github.com/LemmyNet/lemmy-ui/issues/203)
- Fix preview description html. Fixes [#110](https://github.com/LemmyNet/lemmy-ui/issues/110)
- Update darkly, make danger darker. Fixes [#16](https://github.com/LemmyNet/lemmy-ui/issues/16)
- Always show previous paginator, extract paginator component.
- Use better comment collapse icon, and add text. Fixes [#318](https://github.com/LemmyNet/lemmy-ui/issues/318)
- Fix symbols issue. Fixes [#319](https://github.com/LemmyNet/lemmy-ui/issues/319)
- Don't restore scroll position on page refresh. Fixes [#186](https://github.com/LemmyNet/lemmy-ui/issues/186)
- Insert triple backticks for 'code' button when multiple lines are selected. [(#311](https://github.com/LemmyNet/lemmy-ui/issues/311))
- Adding a comment here placeholder. Fixes [#301](https://github.com/LemmyNet/lemmy-ui/issues/301)
- Fix non-local community and person links. Fixes [#290](https://github.com/LemmyNet/lemmy-ui/issues/290)
- Fix navbar bug. Fixes [#289](https://github.com/LemmyNet/lemmy-ui/issues/289)
- Hide names of mods / admins without priveleges. Fixes [#285](https://github.com/LemmyNet/lemmy-ui/issues/285)
- Adding URL search type. Fixes [#286](https://github.com/LemmyNet/lemmy-ui/issues/286)
- Add a link to joinlemmy on lemmy.ml signup. Fixes [#235](https://github.com/LemmyNet/lemmy-ui/issues/235)
- Fix duped site description. Fixes [#281](https://github.com/LemmyNet/lemmy-ui/issues/281)

### API

- Added `show_new_posts_notifs` boolean to `SaveUserSettings`, and `LocalUserSettings`.
- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.11.0 -> 0.11.3](https://github.com/LemmyNet/lemmy-js-client/compare/0.11.0...0.11.3-rc.4) .

### Federation

- No changes in this release, but there will be many soon.

## Upgrade notes

To upgrade your instance to `0.11.3`, simply follow the instructions in the documentation:

- [Upgrade with manual Docker installation](https://join-lemmy.org/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://join-lemmy.org/docs/en/administration/install_ansible.html)

# Lemmy v0.11.0 Release (2021-04-27)

## Changes

Since our last release this month, we've had [~60](https://github.com/LemmyNet/lemmy/compare/0.10.0...main) commits to Lemmy.

### Lemmy Server

#### Major Changes

- Add option to disable strict allowlist ( [#1486](https://github.com/LemmyNet/lemmy/issues/1486)) [documentation](https://join-lemmy.org/docs/en/federation/administration.html) 
- Add option to limit community creation to admins only ([#1587](https://github.com/LemmyNet/lemmy/issues/1587))
- Many search improvements:
  - Don't search for communities or users when the id is included.
  - Add creator id to search.

#### General

- Adding a user setting to show / hide scores. Fixes [#1503](https://github.com/LemmyNet/lemmy/issues/1503)
- Add option to hide read posts. Fixes [#1561](https://github.com/LemmyNet/lemmy/issues/1561)
- Mark accounts as bot, and hide bot posts/comments
- Adding a short site description, to be used for joinlemmy instance list
- Adding matrix id validation. Fixes [#1520](https://github.com/LemmyNet/lemmy/issues/1520)
- Adding users active monthly for community sort. Fixes [#1527](https://github.com/LemmyNet/lemmy/issues/1527)
- Don't allow zero-space char in display name. Fixes [#1317](https://github.com/LemmyNet/lemmy/issues/1317)
- Adding more rust captcha features. Fixes [#1248](https://github.com/LemmyNet/lemmy/issues/1248)
- Fixing slur filter regex. Fixes [#1593](https://github.com/LemmyNet/lemmy/issues/1593)

#### API

- Added `ChangePassword` as a separate endpoint from `SaveUserSettings`
- No other breaking changes, but many fields that were previously required are now optional.
- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.10.0 -> 0.11.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.10.0...0.11.0-rc.13) .

#### Federation

- Implement federated bans fixes [#1298](https://github.com/LemmyNet/lemmy/issues/1298)
- Remote mods can update/delete/undelete communities.

### Lemmy UI

- Updating translations.
- Add UI version to UI via docker. Fixes [#263](https://github.com/LemmyNet/lemmy-ui/issues/263)
- Add Korean language
- Add check for unused languages in update_translations.sh
- Validate matrix id on the front end. Fixes [#245](https://github.com/LemmyNet/lemmy-ui/issues/245)
- Communities page sorts by monthly active users. Fixes [#244](https://github.com/LemmyNet/lemmy-ui/issues/244)
- Correctly render HTML in popup notifications
- Fix html notif bug. Fixes [#254](https://github.com/LemmyNet/lemmy-ui/issues/254)
- Fixing issue with debounce. Fixes [#236](https://github.com/LemmyNet/lemmy-ui/issues/236)

## Upgrade notes

### Servers

If you'd like to make a DB backup before upgrading, follow [this guide](https://join-lemmy.org/docs/en/administration/backup_and_restore.html).

To upgrade your instance to `v0.10.0`, simply follow the instructions in the documentation:

- [Upgrade with manual Docker installation](https://join-lemmy.org/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://join-lemmy.org/docs/en/administration/install_ansible.html)

### Clients / Apps

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.10.0 -> 0.11.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.10.0...0.11.0-rc.13) .

# Lemmy v0.10.3 Release (2021-04-07)

- Fixing instances page.
- Fixed unban not working.
- Fixed post title fetching and cross-post search.
- Fixed navigating to a user page.

# Lemmy v0.10.2 Release (2021-04-05)

- Forcing a crash if config.hjson fails to load. Should show errors easier.

# Lemmy v0.10.0 Release (2021-04-05)

## Changes

Since our last release in February, we've had [~150](https://github.com/LemmyNet/lemmy/compare/0.9.9...main) commits to Lemmy. The biggest changes, as we'll outline below, are a split of Lemmy's user tables into federated and local tables, necessitating a `v3` of Lemmy's API, federated moderation, i18n support in join-lemmy.org, and lots of back-end cleanup.

### Lemmy Server

#### General

- Rewrote config implementation, finally allowing us to use newer Rust versions.
- Removed categories. 
- Various refactors.

#### API

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.9.9 -> 0.10.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.9.9...0.10.0-rc.13) .
- Login invalidation on password change, thanks to @Mart-Bogdan

#### Federation

- It is now possible to add users from other instances as community mods.
- Federating Matrix ID.
- Many changes for better compatibility with ActivityPub standard.

#### Database 

- Split the `user_` into `person` and `local_user` tables.
- Strictly typed commonly used ID columns, to prevent DB errors using `i32` as ids.
- Strictly typed URL fields, thanks to ajyoon.
- Created default DB forms, now used in all the unit tests.

### Lemmy UI

- Now using utf-8 emojis.
- Support for all the above changes to Lemmy.
- Typescript-safe i18n strings, thanks to @shilangyu.
- Added expandable post text (click on open book icon).
- Prettier cross-posting, which does smart quoting.
- Bugfixes for restoring scroll position on post page, custom site favicons, and autocomplete for login fields.

### Lemmy Docs

- Gazconroy built an [Async API spec for Lemmy](https://join-lemmy.org/api/index.html), that now serves as our main API docs.

### join-lemmy.org

- Rewrote in inferno isomorphic, added i18n support via [weblate](https://weblate.yerbamate.ml/projects/lemmy/joinlemmy/).
- Added a section on the support page thanking contributors.
- Changed some page urls / titles

## Upgrade notes

**Important**: there are multiple breaking changes:

- Configuration via environment variables is not supported anymore, you must have all your config in the [lemmy.hjson](https://github.com/LemmyNet/lemmy/blob/main/ansible/templates/config.hjson) file ( except for `LEMMY_CONFIG_LOCATION` ).
- The config format for `allowed_instances` and `blocked_instances` has changed, and you need to adjust your config file manually:
    - before: `allowed_instances: ds9.lemmy.ml,enterprise.lemmy.ml`
    - now: `allowed_instances: ["ds9.lemmy.ml", "enterprise.lemmy.ml"]` , and only one of the `allowed_instances` or `blocked_instances` blocks can be set.
- The API has been upgraded from `v2` to `v3`, so all clients need to be updated: [lemmy-js-client: 0.9.9 -> 0.10.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.9.9...0.10.0-rc.13) .

If you'd like to make a DB backup before upgrading, follow [this guide](https://join-lemmy.org/docs/en/administration/backup_and_restore.html).

To upgrade your instance to `v0.10.0`, simply follow the instructions in the documentation:

- [Upgrade with manual Docker installation](https://join-lemmy.org/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://join-lemmy.org/docs/en/administration/install_ansible.html)


## Compilation time

|| v0.9.0 (Rust 1.47) | v0.10.0 (Rust 1.47) | v0.10.0 (Rust 1.51) |
|-| -------- | -------- | -------- |
|Clean | 140s     | 146s     | 119s     |
| Incremental | 28s | 22s | 19s |

Despite ongoing efforts to speed up compilation, it has actually gotten slower when comparing with the same Rust version. Only thanks to improvements in newer Rust versions has our build process gotten faster. This could be simply because we added more code, while Lemmy v0.9.0 had 22.4k lines of Rust, v0.10.0 has 23.8k (an increase of 6%).

v0.9.0 build graph:
![](https://lemmy.ml/pictrs/image/GVBqFnrLqG.jpg)

v0.10.0 build graph:
![](https://lemmy.ml/pictrs/image/NllzjVEyNK.jpg)

We extracted the crates `lemmy_api_crud` and `lemmy_apub_receive` from `lemmy_api` and `lemmy_apub`, respectively, and renamed `lemmy_structs` to `lemmy_api_common`. In the second graph you can see how parts of the api and apub crates are now built nicely in parallel, speeding up builds on multi-core systems.

On the other hand, some crates have gotten much slower to compile, in particular `lemmy_db_queries` (6.5s slower), `lemmy_apub` (6.5s slower if we include `lemmy_apub_receive`). And `lemmy_db_views` is quite slow, just as before.

# Lemmy v0.9.9 Release (2021-02-19)

## Changes

### Lemmy backend
- Added an federated activity query sorting order.
- Explicitly marking posts and comments as public.
- Added a `NewComment` / forum sort for posts.
- Fixed an issue with not setting correct published time for fetched posts.
- Fixed an issue with an open docker port on lemmy-ui.
- Using lemmy post link for RSS link.
- Fixed reason and display name lengths to use char counts instead.

### Lemmy-ui

- Updated translations.
- Made websocket host configurable.
- Added some accessibility features.
- Always showing password reset link.

# Lemmy v0.9.7 Release (2021-02-08)

## Changes

- Posts and comments are no longer live-sorted (meaning most content should stay in place).
- Fixed an issue with the create post title field not expanding when copied from iframely suggestion.
- Fixed broken federated community paging / sorting.
- Added aria attributes for accessibility, thx to @Mitch Lillie.
- Updated translations and added croatian.
- No changes to lemmy back-end.

# Lemmy v0.9.6 Release (2021-02-05)

## Changes

- Fixed inbox_urls not being correctly set, which broke federation in `v0.9.5`. Added some logging to catch these.
- Fixing community search not using auth.
- Moved docs to https://join-lemmy.org
- Fixed an issue w/ lemmy-ui with forms being cleared out.

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

If you'd like to make a DB backup before upgrading, follow [this guide](https://join-lemmy.org/docs/en/administration/backup_and_restore.html).

- [Upgrade with manual Docker installation](https://join-lemmy.org/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://join-lemmy.org/docs/en/administration/install_ansible.html)

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
- Documentation for [admins](https://join-lemmy.org/docs/administration_federation.html) and [devs](https://join-lemmy.org/docs/contributing_federation_overview.html) on how federation works
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

- [with manual Docker installation](https://join-lemmy.org/docs/administration_install_docker.html#updating)
- [with Ansible installation](https://join-lemmy.org/docs/administration_install_ansible.html)

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
[documentation](https://join-lemmy.org/docs/administration_backup_and_restore.html)
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
