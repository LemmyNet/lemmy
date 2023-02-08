# Lemmy v0.17.1 Release (2023-02-03)

## Bugfixes

### Lemmy

- Fixing admin application email subject. Fixes [#2688](https://github.com/LemmyNet/lemmy/issues/2688) ([#2695](https://github.com/LemmyNet/lemmy/issues/2695))
- Fixing person block views. Fixes [#2693](https://github.com/LemmyNet/lemmy/issues/2693) ([#2694](https://github.com/LemmyNet/lemmy/issues/2694))
- Fixing GetPosts active sort index. Fixes [#2683](https://github.com/LemmyNet/lemmy/issues/2683) ([#2684](https://github.com/LemmyNet/lemmy/issues/2684))
- Publish without verify ([#2681](https://github.com/LemmyNet/lemmy/issues/2681))
- Fix paths in release script, update crate versions ([#2680](https://github.com/LemmyNet/lemmy/issues/2680))

### Lemmy-UI

- Fix comment box closing. Fixes [#904](https://github.com/LemmyNet/lemmy-ui/issues/904) ([#914](https://github.com/LemmyNet/lemmy-ui/issues/914))
- Fix showing crosspost dupes. Fixes [#900](https://github.com/LemmyNet/lemmy-ui/issues/900) ([#912](https://github.com/LemmyNet/lemmy-ui/issues/912))
- Fix live updating postres edit. Fixes [#908](https://github.com/LemmyNet/lemmy-ui/issues/908) ([#911](https://github.com/LemmyNet/lemmy-ui/issues/911))
- Removing extra themes. Fixes [#905](https://github.com/LemmyNet/lemmy-ui/issues/905) ([#910](https://github.com/LemmyNet/lemmy-ui/issues/910))
- Fixing post setState error. Fixes [#902](https://github.com/LemmyNet/lemmy-ui/issues/902) ([#903](https://github.com/LemmyNet/lemmy-ui/issues/903))

# Lemmy v0.17.0 Release (2023-01-31)

## What is Lemmy?

Lemmy is a self-hosted social link aggregation and discussion platform. It is completely free and open, and not controlled by any company. This means that there is no advertising, tracking, or secret algorithms. Content is organized into communities, so it is easy to subscribe to topics that you are interested in, and ignore others. Voting is used to bring the most interesting items to the top.

## Major Changes

### Language Tags

Content can now be tagged to indicate the language it is written in. These tags can be used to filter content, so that you only see posts in languages which you actually understand. Instances and communities can also specify which languages are allowed, and prevent posting in other languages.

In the future this will also allow for integrated translation tools.

### Comment trees

Lemmy has changed the way it stores comments, in order to be able to properly limit the comments shown to a maximum depth.

Included are proper comment links (`/comment/id`), where you can see its children, a count of its hidden children, and a _context_ button to view its parents, or the post.

### Featured posts

Admins and mods can now "feature" (this used to be called "sticky" ala reddit) posts to the top of either a community, or the top of the front page. This makes possible announcement and bulletin-type posts.

Special thanks to @makotech for adding this feature.

### Federation

Lemmy users can now be followed. Just visit a user profile from another platform like Mastodon, and click the follow button, then you will receive new posts and comments in the timeline.

Votes are now federated as private. This prevents other platforms from showing who voted on a given post, and it also means that Lemmy now counts votes from Mastodon.

This release also improves compatibility with Pleroma. If you previously had trouble interacting between Pleroma and Lemmy, give it another try.

We've extracted the main federation logic into its own library, [activitypub-federation-rust](https://github.com/LemmyNet/activitypub-federation-rust). It is open source and can be used by other projects to implement Activitypub federation, without having to reinvent the wheel. The library helps with handling HTTP signatures, sending and receiving activities, fetching remote objects and more.

### Other changes

- Admins can now purge content and pictures from the database.
- Mods can _distinguish_ a comment, "stickying" it to the top of a post. Useful for mod messages and announcements.
- Number of new / unread comments are now shown for each post.
- Lemmy now automatically embeds videos from Peertube, Youtube and other sites which provide an embed link via Opengraph attribute.
- You can give your site "taglines", short markdown messages, which are shown at the top of your front page. Thanks to @makotech for adding this.
- You can now report private messages.
- Most settings have been moved from the config file into the database. This means they can be updated much easier, and apply immediately without a restart.
- When setting up a new Lemmy instance, it doesn't create a default community anymore. Instead this needs to be done manually.
- Admins can choose to receive emails for new registration applications.
- An upgrade of diesel to v2.0, our rust -> postgres layer.
- Too many bugfixes to count, they are listed below.

## Upgrade instructions

This upgrade requires a newer version of postgres, which **must be done manually**. Do not try to use Ansible.

`cd` to your lemmy docker directory and run this helper script:

```
sudo wget https://raw.githubusercontent.com/LemmyNet/lemmy/main/scripts/postgres_12_to_15_upgrade.sh
sudo sh postgres_12_to_15_upgrade.sh
```

This script saves a copy of your old database as `12_15.dump.sql`. **Do not delete this file until you've followed all the instructions below, and the upgrade is complete.**

Next, **manually edit** your [lemmy.hjson](https://github.com/LemmyNet/lemmy/blob/main/config/defaults.hjson) to account for a few breaking changes:

- `pictrs_url` is removed, and the pictrs config is now a block. If using docker, it should look like:
  ```
  pictrs: {
      url: "http://pictrs:8080/"
      # api_key: "API_KEY"
  }
  ```
- The `rate_limit`, `federation`, `captcha`, and `slur_filter` blocks should be removed, as they are now in the database, and can be updated through the UI.
- The site setup has removed a few fields.
- See the link above for every setting.

Next, edit your `docker-compose.yml` file to use the newer version of lemmy and lemmy-ui.

The `image` lines should look like:

- `image: richardarpanet/lemmy:0.17.0` for lemmy
- `image: dessalines/lemmy-ui:0.17.0` for lemmy-ui
- The `lemmy-ui` environment variables have changed, and should now look like:
  ```
    environment:
      - LEMMY_UI_LEMMY_INTERNAL_HOST=lemmy:8536
      - LEMMY_UI_LEMMY_EXTERNAL_HOST={{ domain }}
      - LEMMY_UI_HTTPS=true
  ```
- You can always find the latest version [here](https://github.com/LemmyNet/lemmy-ansible/blob/main/VERSION).
- Ensure that postgres is `postgres:15-alpine` (the upgrade script above should have already set this correctly)

If you're having any problems, your docker-compose.yml should look similar to [the one from the lemmy-ansible repo](https://github.com/LemmyNet/lemmy-ansible/blob/main/templates/docker-compose.yml).

Finally, run `sudo docker-compose up -d`, and wait for lemmy to start up.

_Note_: On production databases with thousands of comments, this upgrade **takes several hours**. If your system has problems upgrading, or you'd like to speed up the upgrade, consider tuning your postgres database using the [instructions here](https://github.com/LemmyNet/lemmy/blob/main/docker/dev/docker-compose.yml#L88). If not, just wait for the database migrations to complete, as this large migration of the `comment` table only ever needs to be run once.

_Note_: If you have any issues upgrading, you can restore your old database using the [backup and restore instructions here](https://join-lemmy.org/docs/en/administration/backup_and_restore.html).

If you need help with the upgrade, you can ask in our [support forum](https://lemmy.ml/c/lemmy_support) or on the [Matrix Chat](https://matrix.to/#/!BZVTUuEiNmRcbFeLeI:matrix.org).

## Support development

We (@dessalines and @nutomic) have been working full-time on Lemmy for almost three years. This is largely thanks to support from [NLnet foundation](https://nlnet.nl/).

If you like using Lemmy, and want to make sure that we will always be available to work full time building it, consider [donating to support its development](https://join-lemmy.org/donate). No one likes recurring donations, but they've proven to be the only way that open-source software like Lemmy can stay independent and alive.

## Changes

### API

- [lemmy-js-client 0.16.4 -> 0.17.0 API changes](https://github.com/LemmyNet/lemmy-js-client/compare/0.16.4...0.17.0-rc.62)

### Config

- [lemmy.hjson](https://github.com/LemmyNet/lemmy/blob/main/config/defaults.hjson)

### Lemmy Server

- Speeding up comment-ltree migration, fixing index creation. Fixes [#2664](https://github.com/LemmyNet/lemmy/issues/2664)
- Add feature to embed pictrs in lemmy binary (fixes [#2627](https://github.com/LemmyNet/lemmy/issues/2627)) ([#2633](https://github.com/LemmyNet/lemmy/issues/2633))
- Update post_aggregates indexes to account for featured_local and featured_community columns. ([#2661](https://github.com/LemmyNet/lemmy/issues/2661))
- Post creation from Mastodon (fixes [#2590](https://github.com/LemmyNet/lemmy/issues/2590)) ([#2651](https://github.com/LemmyNet/lemmy/issues/2651))
- Upgrade to postgres 15. ([#2659](https://github.com/LemmyNet/lemmy/issues/2659))
- Add reddit -> lemmy importer to readme. ([#2662](https://github.com/LemmyNet/lemmy/issues/2662))
- Some script improvements ([#2654](https://github.com/LemmyNet/lemmy/issues/2654))
- Use enum for registration mode setting ([#2604](https://github.com/LemmyNet/lemmy/issues/2604))
- Removing sniptt/monads for lemmy-js-client. ([#2644](https://github.com/LemmyNet/lemmy/issues/2644))
- Fix historical post fetching. Fixes [#2640](https://github.com/LemmyNet/lemmy/issues/2640) ([#2643](https://github.com/LemmyNet/lemmy/issues/2643))
- Adding the go client. ([#2629](https://github.com/LemmyNet/lemmy/issues/2629))
- Point to !lemmy_support for support questions ([#2638](https://github.com/LemmyNet/lemmy/issues/2638))
- Add documentation for using Lemmy API from Rust ([#2639](https://github.com/LemmyNet/lemmy/issues/2639))
- Improve application question check ([#2628](https://github.com/LemmyNet/lemmy/issues/2628))
- Fix user following ([#2623](https://github.com/LemmyNet/lemmy/issues/2623))
- Allow embedding Lemmy, fix setup error ([#2618](https://github.com/LemmyNet/lemmy/issues/2618))
- Fixing missing forms, incorrect user discussion_languages ([#2580](https://github.com/LemmyNet/lemmy/issues/2580))
- Add support for Featured Posts ([#2585](https://github.com/LemmyNet/lemmy/issues/2585))
- Remove federation backwards compatibility with 0.16.x ([#2183](https://github.com/LemmyNet/lemmy/issues/2183))
- Rework websocket ([#2598](https://github.com/LemmyNet/lemmy/issues/2598))
- Add SendActivity trait so that api crates compile in parallel with lemmy_apub
- Move code to generate apub urls into lemmy_api_common
- Builds lemmy_routes in parallel with lemmy_apub
- Merge websocket crate into api_common
- Check user accepted before sending jwt in password reset (fixes [#2591](https://github.com/LemmyNet/lemmy/issues/2591)) ([#2597](https://github.com/LemmyNet/lemmy/issues/2597))
- Relax honeypot check (fixes [#2595](https://github.com/LemmyNet/lemmy/issues/2595)) ([#2596](https://github.com/LemmyNet/lemmy/issues/2596))
- Use audience field to federate items in groups (fixes [#2464](https://github.com/LemmyNet/lemmy/issues/2464)) ([#2584](https://github.com/LemmyNet/lemmy/issues/2584))
- Federate group moderators using attributedTo field ([#2588](https://github.com/LemmyNet/lemmy/issues/2588))
- Set cargo home in ci to avoid redownloading deps between steps ([#2587](https://github.com/LemmyNet/lemmy/issues/2587))
- Add some more clippy lints ([#2586](https://github.com/LemmyNet/lemmy/issues/2586))
- Use release docker image for nightly build ([#2583](https://github.com/LemmyNet/lemmy/issues/2583))
- Implement federated user following (fixes [#752](https://github.com/LemmyNet/lemmy/issues/752)) ([#2577](https://github.com/LemmyNet/lemmy/issues/2577))
- Upgrade activitypub_federation to 0.3.4 ([#2581](https://github.com/LemmyNet/lemmy/issues/2581))
- Upgrade activitypub_federation crate to 0.3.3 (ref [#2511](https://github.com/LemmyNet/lemmy/issues/2511)) ([#2578](https://github.com/LemmyNet/lemmy/issues/2578))
- Remove federation settings, rely on sensible defaults instead ([#2574](https://github.com/LemmyNet/lemmy/issues/2574))
- Fix clippy lints. ([#2572](https://github.com/LemmyNet/lemmy/issues/2572))
- Add support for Taglines ([#2548](https://github.com/LemmyNet/lemmy/issues/2548))
- Various pedantic clippy fixes ([#2568](https://github.com/LemmyNet/lemmy/issues/2568))
- Sort vecs before assert to avoid random test failures ([#2569](https://github.com/LemmyNet/lemmy/issues/2569))
- Display build status badge from drone.join-lemmy.org ([#2564](https://github.com/LemmyNet/lemmy/issues/2564))
- Specify dependencies and metadata for entire workspace ([#2565](https://github.com/LemmyNet/lemmy/issues/2565))
- Use enum_delegate crate ([#2554](https://github.com/LemmyNet/lemmy/issues/2554))
- Live reload settings (fixes [#2508](https://github.com/LemmyNet/lemmy/issues/2508)) ([#2543](https://github.com/LemmyNet/lemmy/issues/2543))
- Fix activity list test ([#2562](https://github.com/LemmyNet/lemmy/issues/2562))
- When announcing incoming activities, keep extra fields ([#2550](https://github.com/LemmyNet/lemmy/issues/2550))
- Mobilizon federation ([#2544](https://github.com/LemmyNet/lemmy/issues/2544))
- Update doku dependency for easier formatting of defaults.hjson ([#2553](https://github.com/LemmyNet/lemmy/issues/2553))
- Translated README.md to Chinese ([#2549](https://github.com/LemmyNet/lemmy/issues/2549))
- Add diesel_async, get rid of blocking function ([#2510](https://github.com/LemmyNet/lemmy/issues/2510))
- Use urlencoding for db url params (fixes [#2532](https://github.com/LemmyNet/lemmy/issues/2532)) ([#2537](https://github.com/LemmyNet/lemmy/issues/2537))
- Dont serve apub json for removed objects (ref [#2522](https://github.com/LemmyNet/lemmy/issues/2522)) ([#2538](https://github.com/LemmyNet/lemmy/issues/2538))
- Fix password length check ([#2536](https://github.com/LemmyNet/lemmy/issues/2536))
- Remove explicit panic from db connection code (fixes [#2533](https://github.com/LemmyNet/lemmy/issues/2533)) ([#2535](https://github.com/LemmyNet/lemmy/issues/2535))
- Send error message when rate limit is reached ([#2527](https://github.com/LemmyNet/lemmy/issues/2527))
- Mark own private messages as read in api (fixes [#2484](https://github.com/LemmyNet/lemmy/issues/2484)) ([#2531](https://github.com/LemmyNet/lemmy/issues/2531))
- Mark objects as not deleted when received via apub (fixes [#2507](https://github.com/LemmyNet/lemmy/issues/2507)) ([#2528](https://github.com/LemmyNet/lemmy/issues/2528))
- Group imports dess ([#2526](https://github.com/LemmyNet/lemmy/issues/2526))
- Fix invalid config in docker/prod (fixes [#2520](https://github.com/LemmyNet/lemmy/issues/2520)) ([#2524](https://github.com/LemmyNet/lemmy/issues/2524))
- Fix local site images. ([#2519](https://github.com/LemmyNet/lemmy/issues/2519))
- Fix 2455: Check auth for pictrs when instance is private. ([#2477](https://github.com/LemmyNet/lemmy/issues/2477))
- Fix limit_languages to operate on correct instance (fixes [#2496](https://github.com/LemmyNet/lemmy/issues/2496)) ([#2518](https://github.com/LemmyNet/lemmy/issues/2518))
- Image improvements ([#2513](https://github.com/LemmyNet/lemmy/issues/2513))
- Make verify apub url function async ([#2514](https://github.com/LemmyNet/lemmy/issues/2514))
- Moving settings to Database. ([#2492](https://github.com/LemmyNet/lemmy/issues/2492))
- Enable lto, strip symbols via cargo.toml ([#2512](https://github.com/LemmyNet/lemmy/issues/2512))
- Fix docker dev build ([#2509](https://github.com/LemmyNet/lemmy/issues/2509))
- Federate votes as private ([#2501](https://github.com/LemmyNet/lemmy/issues/2501))
- Dont try to send activities if federation is disabled (fixes [#2499](https://github.com/LemmyNet/lemmy/issues/2499)) ([#2500](https://github.com/LemmyNet/lemmy/issues/2500))
- Return empty vec when reading all languages (fixes [#2495](https://github.com/LemmyNet/lemmy/issues/2495)) ([#2497](https://github.com/LemmyNet/lemmy/issues/2497))
- Update clippy to use Rust 1.64 ([#2498](https://github.com/LemmyNet/lemmy/issues/2498))
- Only allow authenticated users to fetch remote objects ([#2493](https://github.com/LemmyNet/lemmy/issues/2493))
- More real-world prod config, separate lemmy config ([#2487](https://github.com/LemmyNet/lemmy/issues/2487))
- Fix check for federated mod actions ([#2489](https://github.com/LemmyNet/lemmy/issues/2489))
- Make docker-compose more clear and explicit ([#2469](https://github.com/LemmyNet/lemmy/issues/2469))
- implement language tags for site/community in db and api ([#2434](https://github.com/LemmyNet/lemmy/issues/2434))
- Change description in readme ([#2481](https://github.com/LemmyNet/lemmy/issues/2481))
- Use compat mode when signing outgoing activities (fixes [#1984](https://github.com/LemmyNet/lemmy/issues/1984)) ([#2473](https://github.com/LemmyNet/lemmy/issues/2473))
- Check to make sure comment isnt deleted / removed for unread count. ([#2472](https://github.com/LemmyNet/lemmy/issues/2472))
- Dont show deleted users or communities on profile page. ([#2450](https://github.com/LemmyNet/lemmy/issues/2450))
- Adding email admins for new applications. Fixes [#2271](https://github.com/LemmyNet/lemmy/issues/2271) ([#2390](https://github.com/LemmyNet/lemmy/issues/2390))
- Showing # of unread comments for posts. Fixes [#2134](https://github.com/LemmyNet/lemmy/issues/2134) ([#2393](https://github.com/LemmyNet/lemmy/issues/2393))
- Convert emails to lowercase (fixes [#2462](https://github.com/LemmyNet/lemmy/issues/2462)) ([#2463](https://github.com/LemmyNet/lemmy/issues/2463))
- Remove unnecessary show_deleted_and_removed comments for a profile. ([#2458](https://github.com/LemmyNet/lemmy/issues/2458))
- Remove pointless language joins. ([#2451](https://github.com/LemmyNet/lemmy/issues/2451))
- Fix rate limit error messages. Fixes [#2428](https://github.com/LemmyNet/lemmy/issues/2428) ([#2449](https://github.com/LemmyNet/lemmy/issues/2449))
- Fix missing local user from post queries. ([#2447](https://github.com/LemmyNet/lemmy/issues/2447))
- Diesel 2.0.0 upgrade ([#2452](https://github.com/LemmyNet/lemmy/issues/2452))
- Allow filtering out of deleted and removed comments when getting person details ([#2446](https://github.com/LemmyNet/lemmy/issues/2446))
- Implement reports for private messages ([#2433](https://github.com/LemmyNet/lemmy/issues/2433))
- Check for slurs in account creation. ([#2443](https://github.com/LemmyNet/lemmy/issues/2443))
- The language id is crucial for front ends. ([#2437](https://github.com/LemmyNet/lemmy/issues/2437))
- Update docker version to 0.16.6. Fixes [#2435](https://github.com/LemmyNet/lemmy/issues/2435) ([#2438](https://github.com/LemmyNet/lemmy/issues/2438))
- Adding job to drop phantom ccnew indexes. Fixes [#2431](https://github.com/LemmyNet/lemmy/issues/2431) ([#2432](https://github.com/LemmyNet/lemmy/issues/2432))
- Don't search for community descriptions, search for user display_name. ([#2430](https://github.com/LemmyNet/lemmy/issues/2430))
- Increase default search rate limit. ([#2424](https://github.com/LemmyNet/lemmy/issues/2424))
- fix clippy
- dont set default user languages in api code (already done in db)
- dont test with all features
- clippy fixes
- api changes for comment language tagging
- add test for comment view languages
- fix tests
- Add language tags for comments
- Pass LocalUser to PostQuery etc, instead of separate params ([#2413](https://github.com/LemmyNet/lemmy/issues/2413))
- Tag posts and comments with language (fixes [#440](https://github.com/LemmyNet/lemmy/issues/440)) ([#2269](https://github.com/LemmyNet/lemmy/issues/2269))
- Rejected federated pm from blocked users (fixes [#2398](https://github.com/LemmyNet/lemmy/issues/2398)) ([#2408](https://github.com/LemmyNet/lemmy/issues/2408))
- Adding distinguish comment. Fixes [#2002](https://github.com/LemmyNet/lemmy/issues/2002) ([#2391](https://github.com/LemmyNet/lemmy/issues/2391))
- Fix pictrs routing ([#2407](https://github.com/LemmyNet/lemmy/issues/2407))
- Add postgres auto-explain for dev testing. ([#2399](https://github.com/LemmyNet/lemmy/issues/2399))
- Add Modlog Filters ([#2313](https://github.com/LemmyNet/lemmy/issues/2313))
- Accept Image objects in attachments ([#2394](https://github.com/LemmyNet/lemmy/issues/2394))
- Tweaking postgres upgrade script ([#2389](https://github.com/LemmyNet/lemmy/issues/2389))
- Use typed-builder crate for queries ([#2379](https://github.com/LemmyNet/lemmy/issues/2379))
- Use doku release version (ref [#2343](https://github.com/LemmyNet/lemmy/issues/2343)) ([#2386](https://github.com/LemmyNet/lemmy/issues/2386))
- First pass at adding comment trees. ([#2362](https://github.com/LemmyNet/lemmy/issues/2362))
- Update apub examples to remove `to` field (ref [#2380](https://github.com/LemmyNet/lemmy/issues/2380)) ([#2382](https://github.com/LemmyNet/lemmy/issues/2382))
- Handle Like, Undo/Like activities from Mastodon, add tests (fixes [#2378](https://github.com/LemmyNet/lemmy/issues/2378)) ([#2380](https://github.com/LemmyNet/lemmy/issues/2380))
- Fix a few form options for diesel. Fixes [#2287](https://github.com/LemmyNet/lemmy/issues/2287) ([#2376](https://github.com/LemmyNet/lemmy/issues/2376))
- Remove docker/pleroma/ folder ([#2381](https://github.com/LemmyNet/lemmy/issues/2381))
- Remove listing type community. Fixes [#2361](https://github.com/LemmyNet/lemmy/issues/2361) ([#2377](https://github.com/LemmyNet/lemmy/issues/2377))
- Dont allow login if account is banned or deleted (fixes [#2372](https://github.com/LemmyNet/lemmy/issues/2372)) ([#2374](https://github.com/LemmyNet/lemmy/issues/2374))
- Send websocket message on accepted follow. Fixes [#2369](https://github.com/LemmyNet/lemmy/issues/2369) ([#2375](https://github.com/LemmyNet/lemmy/issues/2375))
- Fix panics in search_by_apub_id() (fixes [#2371](https://github.com/LemmyNet/lemmy/issues/2371)) ([#2373](https://github.com/LemmyNet/lemmy/issues/2373))
- Fix follow being stuck as pending after accept ([#2366](https://github.com/LemmyNet/lemmy/issues/2366))
- Adding 0.16.6 release
- Change config pictrs key name ([#2360](https://github.com/LemmyNet/lemmy/issues/2360))
- Config changes, remove unused image purge function ([#2343](https://github.com/LemmyNet/lemmy/issues/2343))
- Fix problem where actors can have empty public key (fixes [#2347](https://github.com/LemmyNet/lemmy/issues/2347)) ([#2348](https://github.com/LemmyNet/lemmy/issues/2348))
- remove parking_lot ([#2350](https://github.com/LemmyNet/lemmy/issues/2350))
- Revert "Use correct url for activitystreams protocol context ([#2326](https://github.com/LemmyNet/lemmy/issues/2326))" ([#2351](https://github.com/LemmyNet/lemmy/issues/2351))
- Adding check for requests with no id or name, adding max limit. ([#2265](https://github.com/LemmyNet/lemmy/issues/2265))
- Dont allow blocking admin ([#2340](https://github.com/LemmyNet/lemmy/issues/2340))
- Fix wrong clippy warning in ci ([#2339](https://github.com/LemmyNet/lemmy/issues/2339))
- Be more explicit about returning deleted actors or not ([#2335](https://github.com/LemmyNet/lemmy/issues/2335))
- Specify minimum Rust version 1.57 (fixes [#2333](https://github.com/LemmyNet/lemmy/issues/2333)) ([#2334](https://github.com/LemmyNet/lemmy/issues/2334))
- Remove update and read site config. Fixes [#2306](https://github.com/LemmyNet/lemmy/issues/2306) ([#2329](https://github.com/LemmyNet/lemmy/issues/2329))
- Don't create or initially follow a default community. Fixes [#2317](https://github.com/LemmyNet/lemmy/issues/2317) ([#2328](https://github.com/LemmyNet/lemmy/issues/2328))
- Increase RSS fetch limit to 20. Fixes [#2319](https://github.com/LemmyNet/lemmy/issues/2319) ([#2327](https://github.com/LemmyNet/lemmy/issues/2327))
- Expose pending 2 ([#2282](https://github.com/LemmyNet/lemmy/issues/2282))
- Use correct url for activitystreams protocol context ([#2326](https://github.com/LemmyNet/lemmy/issues/2326))
- Move setting http_fetch_retry_limit into federation block ([#2314](https://github.com/LemmyNet/lemmy/issues/2314))
- Fix length of post_report.original_post_name db field (fixes [#2311](https://github.com/LemmyNet/lemmy/issues/2311)) ([#2315](https://github.com/LemmyNet/lemmy/issues/2315))
- Adding admin purging of DB items and pictures. [#904](https://github.com/LemmyNet/lemmy/issues/904) [#1331](https://github.com/LemmyNet/lemmy/issues/1331) ([#1809](https://github.com/LemmyNet/lemmy/issues/1809))
- Fix: Use correctly parseable JSON-LD context ([#2299](https://github.com/LemmyNet/lemmy/issues/2299))
- Fix lemmy version in prod docker-compose.yml ([#2304](https://github.com/LemmyNet/lemmy/issues/2304))
- Upgrade activitypub_federation to 0.2.0, add setting federation.debug ([#2300](https://github.com/LemmyNet/lemmy/issues/2300))
- Remove unused setup config vars ([#2302](https://github.com/LemmyNet/lemmy/issues/2302))
- Add pub use for db crates in api_common ([#2305](https://github.com/LemmyNet/lemmy/issues/2305))
- Add link to Matrix chat in readme ([#2303](https://github.com/LemmyNet/lemmy/issues/2303))
- Accept private like ([#1968](https://github.com/LemmyNet/lemmy/issues/1968)) ([#2301](https://github.com/LemmyNet/lemmy/issues/2301))
- Move different features drone check to below defaults.hjson check. ([#2296](https://github.com/LemmyNet/lemmy/issues/2296))
- Bump lettre to 0.10.0-rc.7 ([#2297](https://github.com/LemmyNet/lemmy/issues/2297))
- Remove unused cargo.toml files ([#2293](https://github.com/LemmyNet/lemmy/issues/2293))
- Forbid outgoing requests in activitypub tests (fixes [#2289](https://github.com/LemmyNet/lemmy/issues/2289)) ([#2294](https://github.com/LemmyNet/lemmy/issues/2294))
- Embed Peertube videos ([#2261](https://github.com/LemmyNet/lemmy/issues/2261))
- Run cargo check for each crate with different features (ref [#2284](https://github.com/LemmyNet/lemmy/issues/2284)) ([#2292](https://github.com/LemmyNet/lemmy/issues/2292))
- Remove 0.15 federation compat code ([#2131](https://github.com/LemmyNet/lemmy/issues/2131))
- Extract Activitypub logic into separate library ([#2288](https://github.com/LemmyNet/lemmy/issues/2288))

### Lemmy UI

- Fixing requireapplication string. ([#895](https://github.com/LemmyNet/lemmy-ui/issues/895))
- Fixing PWA install. Fixes [#822](https://github.com/LemmyNet/lemmy-ui/issues/822) ([#893](https://github.com/LemmyNet/lemmy-ui/issues/893))
- Removing monads. Fixes [#884](https://github.com/LemmyNet/lemmy-ui/issues/884) ([#886](https://github.com/LemmyNet/lemmy-ui/issues/886))
- Sanitize article html. Fixes [#882](https://github.com/LemmyNet/lemmy-ui/issues/882) ([#883](https://github.com/LemmyNet/lemmy-ui/issues/883))
- Add `id` to `App` component ([#880](https://github.com/LemmyNet/lemmy-ui/issues/880))
- Adding Community Language fixes. [#783](https://github.com/LemmyNet/lemmy-ui/issues/783) ([#868](https://github.com/LemmyNet/lemmy-ui/issues/868))
- Add FeaturedPost Support ([#873](https://github.com/LemmyNet/lemmy-ui/issues/873))
- Fix csp header for svgs in firefox. Fixes [#869](https://github.com/LemmyNet/lemmy-ui/issues/869) ([#870](https://github.com/LemmyNet/lemmy-ui/issues/870))
- Remove federation strict_allowlist and retry_count. ([#867](https://github.com/LemmyNet/lemmy-ui/issues/867))
- Add Taglines support ([#854](https://github.com/LemmyNet/lemmy-ui/issues/854))
- Fix wrong comment link. Fixes [#714](https://github.com/LemmyNet/lemmy-ui/issues/714) ([#865](https://github.com/LemmyNet/lemmy-ui/issues/865))
- Dont render images in tippy. Fixes [#776](https://github.com/LemmyNet/lemmy-ui/issues/776) ([#864](https://github.com/LemmyNet/lemmy-ui/issues/864))
- Move symbols to its own cacheable file. Fixes [#809](https://github.com/LemmyNet/lemmy-ui/issues/809) ([#862](https://github.com/LemmyNet/lemmy-ui/issues/862))
- Hide post report images. Fixes [#824](https://github.com/LemmyNet/lemmy-ui/issues/824) ([#861](https://github.com/LemmyNet/lemmy-ui/issues/861))
- Add inline markdown rendering for post titles. Fixes [#827](https://github.com/LemmyNet/lemmy-ui/issues/827) ([#860](https://github.com/LemmyNet/lemmy-ui/issues/860))
- Show deleted on profile page. Fixes [#834](https://github.com/LemmyNet/lemmy-ui/issues/834) ([#859](https://github.com/LemmyNet/lemmy-ui/issues/859))
- Make sure user is logged in for site creation. Fixes [#838](https://github.com/LemmyNet/lemmy-ui/issues/838) ([#858](https://github.com/LemmyNet/lemmy-ui/issues/858))
- Fix missing report shield. Fixes [#842](https://github.com/LemmyNet/lemmy-ui/issues/842) ([#855](https://github.com/LemmyNet/lemmy-ui/issues/855))
- Increase markdown field char limit to 50k. Fixes [#849](https://github.com/LemmyNet/lemmy-ui/issues/849) ([#850](https://github.com/LemmyNet/lemmy-ui/issues/850))
- Adding new site setup fields. ([#840](https://github.com/LemmyNet/lemmy-ui/issues/840))
- Fix workaround for broken logout ([#836](https://github.com/LemmyNet/lemmy-ui/issues/836))
- Strip html from og descriptions. Fixes [#830](https://github.com/LemmyNet/lemmy-ui/issues/830) ([#831](https://github.com/LemmyNet/lemmy-ui/issues/831))
- Cleanup docker builds ([#829](https://github.com/LemmyNet/lemmy-ui/issues/829))
- Fix admin default listing type. Fixes [#797](https://github.com/LemmyNet/lemmy-ui/issues/797) ([#818](https://github.com/LemmyNet/lemmy-ui/issues/818))
- Search button and input style fixes ([#825](https://github.com/LemmyNet/lemmy-ui/issues/825))
- Support new video embed api format (fixes [#709](https://github.com/LemmyNet/lemmy-ui/issues/709)) ([#817](https://github.com/LemmyNet/lemmy-ui/issues/817))
- Change for container divs to container-lg ([#813](https://github.com/LemmyNet/lemmy-ui/issues/813))
- Remove newline, save space for toast message.
- Avoid browser warning about leaving page, handle delete image fail and add user filenames to messages.
- Avoid browser warning about leaving page, handle delete image fail.
- Adding private message reporting. Fixes [#782](https://github.com/LemmyNet/lemmy-ui/issues/782) ([#806](https://github.com/LemmyNet/lemmy-ui/issues/806))
- Adding the email_admins for new application config. ([#742](https://github.com/LemmyNet/lemmy-ui/issues/742))
- Adding new unread comments. ([#749](https://github.com/LemmyNet/lemmy-ui/issues/749))
- Fix broken profile page, and missing sidebars. ([#795](https://github.com/LemmyNet/lemmy-ui/issues/795))
- Adding a loading indicator for post community searching. Fixes [#692](https://github.com/LemmyNet/lemmy-ui/issues/692) ([#794](https://github.com/LemmyNet/lemmy-ui/issues/794))
- Fix missing initial load of discussion languages. ([#793](https://github.com/LemmyNet/lemmy-ui/issues/793))
- Fix posts pushed from blocked users/comms. Fixes [#697](https://github.com/LemmyNet/lemmy-ui/issues/697) ([#792](https://github.com/LemmyNet/lemmy-ui/issues/792))
- Adding post and comment language tagging. Fixes [#771](https://github.com/LemmyNet/lemmy-ui/issues/771) ([#781](https://github.com/LemmyNet/lemmy-ui/issues/781))
- Hide create community ([#787](https://github.com/LemmyNet/lemmy-ui/issues/787))
- Increase fetch limit for user and community searches. Fixes [#756](https://github.com/LemmyNet/lemmy-ui/issues/756) ([#773](https://github.com/LemmyNet/lemmy-ui/issues/773))
- Fix private instance setting. Fixes [#769](https://github.com/LemmyNet/lemmy-ui/issues/769) ([#786](https://github.com/LemmyNet/lemmy-ui/issues/786))
- Hide extra comment and post functionality from search page. Fixes [#752](https://github.com/LemmyNet/lemmy-ui/issues/752) ([#788](https://github.com/LemmyNet/lemmy-ui/issues/788))
- Show create post even if not subscribed. Fixes [#768](https://github.com/LemmyNet/lemmy-ui/issues/768) ([#789](https://github.com/LemmyNet/lemmy-ui/issues/789))
- Cantarell for darkly/darkly-red. Fixes [#779](https://github.com/LemmyNet/lemmy-ui/issues/779) ([#784](https://github.com/LemmyNet/lemmy-ui/issues/784))
- Adding mod / admin distinguish. ([#744](https://github.com/LemmyNet/lemmy-ui/issues/744))
- Disable CSP when in debug mode. ([#743](https://github.com/LemmyNet/lemmy-ui/issues/743))
- Reduce search minLength to 1. Fixes [#750](https://github.com/LemmyNet/lemmy-ui/issues/750) ([#751](https://github.com/LemmyNet/lemmy-ui/issues/751))
- Add support for filtering mod logs
- Documenting and changing a few env vars. Fixes [#661](https://github.com/LemmyNet/lemmy-ui/issues/661) ([#739](https://github.com/LemmyNet/lemmy-ui/issues/739))
- Fixing post_view glitch. Fixes [#740](https://github.com/LemmyNet/lemmy-ui/issues/740) ([#741](https://github.com/LemmyNet/lemmy-ui/issues/741))
- Change CSP rule for connect-src (websocket) to wildcard (fixes [#730](https://github.com/LemmyNet/lemmy-ui/issues/730)) ([#737](https://github.com/LemmyNet/lemmy-ui/issues/737))
- Comment Tree paging ([#726](https://github.com/LemmyNet/lemmy-ui/issues/726))
- Adding block from community sidebar. Fixes [#690](https://github.com/LemmyNet/lemmy-ui/issues/690) ([#716](https://github.com/LemmyNet/lemmy-ui/issues/716))
- Fix suggested post title html. Fixes [#691](https://github.com/LemmyNet/lemmy-ui/issues/691) ([#717](https://github.com/LemmyNet/lemmy-ui/issues/717))
- Fix missing deny button. Fixes [#723](https://github.com/LemmyNet/lemmy-ui/issues/723) ([#728](https://github.com/LemmyNet/lemmy-ui/issues/728))
- Fix community filtering. ([#729](https://github.com/LemmyNet/lemmy-ui/issues/729))
- use zh-TW for language code, instead of zh_Hant. ([#725](https://github.com/LemmyNet/lemmy-ui/issues/725))
- Fix Notification browser fetch
- Forgot to type a few Searches. Fixes [#718](https://github.com/LemmyNet/lemmy-ui/issues/718) ([#722](https://github.com/LemmyNet/lemmy-ui/issues/722))
- Fixing linkify GC crash. ([#715](https://github.com/LemmyNet/lemmy-ui/issues/715))
- New communities fetch limit is 50. ([#711](https://github.com/LemmyNet/lemmy-ui/issues/711))
- Clicking "subscribe pending" button performs unsubscribe (fixes [#705](https://github.com/LemmyNet/lemmy-ui/issues/705)) ([#706](https://github.com/LemmyNet/lemmy-ui/issues/706))
- Fix site setup and login. Fixes [#699](https://github.com/LemmyNet/lemmy-ui/issues/699) ([#702](https://github.com/LemmyNet/lemmy-ui/issues/702))
- Adding purging of comments, posts, communities, and users. ([#459](https://github.com/LemmyNet/lemmy-ui/issues/459))
- Removing save and read config hjson. Fixes [#695](https://github.com/LemmyNet/lemmy-ui/issues/695) ([#696](https://github.com/LemmyNet/lemmy-ui/issues/696))
- Expose pending 2 ([#662](https://github.com/LemmyNet/lemmy-ui/issues/662))
- Adding option types 2 ([#689](https://github.com/LemmyNet/lemmy-ui/issues/689))
- Fix NPE during new site startup ([#677](https://github.com/LemmyNet/lemmy-ui/issues/677))
- Fixing CSP for iOS devices. Fixes [#669](https://github.com/LemmyNet/lemmy-ui/issues/669) ([#678](https://github.com/LemmyNet/lemmy-ui/issues/678))
- Fix issue with new notification trying to do a fetch.

# Lemmy v0.16.7 Release : Bug fixes (2022-09-14)

_Written by @dessalines and @nutomic, 2022-09-14_

A few bug fixes:

- Fix missing auth on new post refresh. ([#764](https://github.com/LemmyNet/lemmy-ui/issues/764))
- Change CSP rule for connect-src (websocket) to wildcard (fixes [#730](https://github.com/LemmyNet/lemmy-ui/issues/730)) ([#737](https://github.com/LemmyNet/lemmy-ui/issues/737))
- Increase default search rate limit. ([#2424](https://github.com/LemmyNet/lemmy/issues/2424))
- Rejected federated pm from blocked users (fixes [#2398](https://github.com/LemmyNet/lemmy/issues/2398)) ([#2408](https://github.com/LemmyNet/lemmy/issues/2408))
- Handle Like, Undo/Like activities from Mastodon, add tests (fixes [#2378](https://github.com/LemmyNet/lemmy/issues/2378)) ([#2380](https://github.com/LemmyNet/lemmy/issues/2380))
- Dont allow login if account is banned or deleted (fixes [#2372](https://github.com/LemmyNet/lemmy/issues/2372)) ([#2374](https://github.com/LemmyNet/lemmy/issues/2374))
- Fix panics in search_by_apub_id() (fixes [#2371](https://github.com/LemmyNet/lemmy/issues/2371)) ([#2373](https://github.com/LemmyNet/lemmy/issues/2373))

# Lemmy v0.16.6 Release : bug fixes (2022-07-19)

A few bug fixes:

- Fix problem where actors can have empty public key (fixes [#2347](https://github.com/LemmyNet/lemmy/issues/2347)) ([#2348](https://github.com/LemmyNet/lemmy/issues/2348))
- Be more explicit about returning deleted actors or not ([#2335](https://github.com/LemmyNet/lemmy/issues/2335))
- Dont allow blocking admin ([#2340](https://github.com/LemmyNet/lemmy/issues/2340))
- Increase RSS fetch limit to 20. Fixes [#2319](https://github.com/LemmyNet/lemmy/issues/2319) ([#2327](https://github.com/LemmyNet/lemmy/issues/2327))
- Fix length of post_report.original_post_name db field (fixes [#2311](https://github.com/LemmyNet/lemmy/issues/2311)) ([#2315](https://github.com/LemmyNet/lemmy/issues/2315))
- Add pub use for db crates in api_common ([#2305](https://github.com/LemmyNet/lemmy/issues/2305))
- Accept private like ([#1968](https://github.com/LemmyNet/lemmy/issues/1968)) ([#2301](https://github.com/LemmyNet/lemmy/issues/2301))

# Lemmy v0.16.4 Release : Peertube federation, Rust API and other improvements (2022-05-27)

## What is Lemmy?

Lemmy is a self-hosted social link aggregation and discussion platform. It is completely free and open, and not controlled by any company. This means that there is no advertising, tracking, or secret algorithms. Content is organized into communities, so it is easy to subscribe to topics that you are interested in, and ignore others. Voting is used to bring the most interesting items to the top.

## Major changes

This version adds a new community setting "restricted". If this is active, only moderators can post in the community (but anyone can comment). This can be useful for announcements or blogs.

In your site settings there is a new field for legal information. This can be used to present terms of service, privacy policy etc.

We've also added an admin setting for the default post listing type. This determines whether users without login, and newly registered users, will see the `Local` or `All` timeline by default.

HTML tags are now disabled in markdown, as the were causing some issues.

### Federation

Lemmy now federates with [Peertube](https://joinpeertube.org/)! Be aware that this requires Peertube [v4.2.0-rc.1](https://github.com/Chocobozzz/PeerTube/releases/tag/v4.2.0-rc.1) or later. You can now follow Peertube channels from Lemmy and comment on videos. If there is other functionality that you would like to see federated, please open an issue (the same goes for federation with other projects).

When browsing remote Lemmy communities, you will now see the site description and rules in the sidebar. Some federated actions did not generate mod log entries previously, this has been fixed. Also, federation with Friendica was approved, Lemmy now correctly receives comments with hashtags. Additionally, the previous version had a check which rejected federation in case the domain of user avatars or banners didn't match the user's domain. This check broke federation with some instances,
and was removed.

### Rust API

If you are thinking of developing a Rust application which interacts with Lemmy, this is now much easier. The [lemmy-api-common](https://crates.io/crates/lemmy_api_common) crate has a new feature which disables all heavy dependencies (like diesel) by default. You can add the crate to your project, and interact with Lemmy API using the exact same structs that Lemmy itself uses. For an example, have a look at [lemmyBB](https://github.com/Nutomic/lemmyBB). Its in a very early stage,
so contributions are welcome!

In other development news, our test instances ([ds9.lemmy.ml](https://ds9.lemmy.ml/), [voyager.lemmy.ml](https://voyager.lemmy.ml/), [enterprise.lemmy.ml](https://enterprise.lemmy.ml/)) are now updated automatically every night with the latest development version. This should make it easier for admins and users to test new features before they are released. At the same time, [join-lemmy.org](https://join-lemmy.org/) and its instance list are also updated automatically every night.

## Upgrade notes

Follow the [Docker or Ansible upgrade instructions here.](https://join-lemmy.org/docs/en/administration/administration.html)

## Support development

We (@dessalines and @nutomic) have been working full-time on Lemmy for almost two years. This is largely thanks to support from [NLnet foundation](https://nlnet.nl/).

If you'd like to support development, and make sure that we will always be available to work full time on Lemmy, consider [donating to support its development](https://join-lemmy.org/donate). We've spent hundreds of hours on Lemmy, and would like to be able to add more developers to our little open-source co-op as time goes on.

## Changes

### API

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.16.0 -> 0.16.4](https://github.com/LemmyNet/lemmy-js-client/compare/0.16.0-rc.1...0.16.4-rc.3) .

### Lemmy

- Add legal information (fixes [#721](https://github.com/LemmyNet/lemmy/issues/721)) ([#2273](https://github.com/LemmyNet/lemmy/issues/2273))
- Add drone task for nightly build ([#2264](https://github.com/LemmyNet/lemmy/issues/2264))
- Fixing malformed rosetta translations. Fixes [#2231](https://github.com/LemmyNet/lemmy/issues/2231)
- Make opentelemetry dependency optional
- Remove check that avatars/banners are locally hosted (fixes [#2254](https://github.com/LemmyNet/lemmy/issues/2254)) ([#2255](https://github.com/LemmyNet/lemmy/issues/2255))
- Simplify building plain/html emails ([#2251](https://github.com/LemmyNet/lemmy/issues/2251))
- Federate with Peertube ([#2244](https://github.com/LemmyNet/lemmy/issues/2244))
- Derive default for api request structs, move type enums ([#2245](https://github.com/LemmyNet/lemmy/issues/2245))
- Add cargo feature for building lemmy_api_common with mininum deps ([#2243](https://github.com/LemmyNet/lemmy/issues/2243))
- Add restricted community field to CreateCommunity, UpdateCommunity (ref [#2235](https://github.com/LemmyNet/lemmy/issues/2235)) ([#2242](https://github.com/LemmyNet/lemmy/issues/2242))
- Implement restricted community (only mods can post) (fixes [#187](https://github.com/LemmyNet/lemmy/issues/187)) ([#2235](https://github.com/LemmyNet/lemmy/issues/2235))
- Update community statistics after post or comment is deleted by user ([#2193](https://github.com/LemmyNet/lemmy/issues/2193))
- Accept comments with hashtags from Friendica ([#2236](https://github.com/LemmyNet/lemmy/issues/2236))
- Remove unused dependencies ([#2239](https://github.com/LemmyNet/lemmy/issues/2239))
- Fix link metadata unit test ([#2237](https://github.com/LemmyNet/lemmy/issues/2237))
- Dont return "admin" for GET user when no id/name is provided (fixes [#1546](https://github.com/LemmyNet/lemmy/issues/1546)) ([#2233](https://github.com/LemmyNet/lemmy/issues/2233))
- Federation: dont overwrite local object from Announce activity ([#2232](https://github.com/LemmyNet/lemmy/issues/2232))
- Require registration application by default ([#2229](https://github.com/LemmyNet/lemmy/issues/2229))
- Add default post listing type (fixes [#2195](https://github.com/LemmyNet/lemmy/issues/2195)) ([#2209](https://github.com/LemmyNet/lemmy/issues/2209))
- Show deny reason to users after a failed login. Fixes [#2191](https://github.com/LemmyNet/lemmy/issues/2191) ([#2206](https://github.com/LemmyNet/lemmy/issues/2206))
- Fix allowlist / blocklist description location. Fixes [#2214](https://github.com/LemmyNet/lemmy/issues/2214) ([#2215](https://github.com/LemmyNet/lemmy/issues/2215))
- Split apart api files ([#2216](https://github.com/LemmyNet/lemmy/issues/2216))
- Changing default listing type to Local from Subscribed.
- Expose remote site info in GetCommunity API (fixes [#2208](https://github.com/LemmyNet/lemmy/issues/2208)) ([#2210](https://github.com/LemmyNet/lemmy/issues/2210))
- Fixing unstable post sorts. Fixes [#2188](https://github.com/LemmyNet/lemmy/issues/2188) ([#2204](https://github.com/LemmyNet/lemmy/issues/2204))
- Adding lemmy_ui_debug for future debug testing. ([#2211](https://github.com/LemmyNet/lemmy/issues/2211))
- Fixing generate unique changeme ([#2205](https://github.com/LemmyNet/lemmy/issues/2205))
- Change Person, Instance types ([#2200](https://github.com/LemmyNet/lemmy/issues/2200))
- Write mod log for federated sticky/lock post actions ([#2203](https://github.com/LemmyNet/lemmy/issues/2203))

### Lemmy-UI

- Adding Legal info ([#666](https://github.com/LemmyNet/lemmy-ui/issues/666))
- Add nightly dev drone cron build. ([#664](https://github.com/LemmyNet/lemmy-ui/issues/664))
- Add LEMMY_UI_CUSTOM_SCRIPT env var. Fixes [#655](https://github.com/LemmyNet/lemmy-ui/issues/655) ([#656](https://github.com/LemmyNet/lemmy-ui/issues/656))
- Turn off html in markdown. Fixes [#650](https://github.com/LemmyNet/lemmy-ui/issues/650) ([#657](https://github.com/LemmyNet/lemmy-ui/issues/657))
- Add posting restricted to mods ([#642](https://github.com/LemmyNet/lemmy-ui/issues/642))
- Add default post listing ([#645](https://github.com/LemmyNet/lemmy-ui/issues/645))
- Don't render markdown for summaries. Fixes [#658](https://github.com/LemmyNet/lemmy-ui/issues/658) ([#659](https://github.com/LemmyNet/lemmy-ui/issues/659))
- Set content security policy http header for all responses ([#621](https://github.com/LemmyNet/lemmy-ui/issues/621))
- Adding site sidebar for remote communities. Fixes [#626](https://github.com/LemmyNet/lemmy-ui/issues/626) ([#640](https://github.com/LemmyNet/lemmy-ui/issues/640))
- Properly debouncing tribute mentions. Fixes [#633](https://github.com/LemmyNet/lemmy-ui/issues/633) ([#639](https://github.com/LemmyNet/lemmy-ui/issues/639))
- Adding litely-red and darkly-red themes. ([#636](https://github.com/LemmyNet/lemmy-ui/issues/636))
- Fixing initial loading of admin page. Fixes [#635](https://github.com/LemmyNet/lemmy-ui/issues/635) ([#638](https://github.com/LemmyNet/lemmy-ui/issues/638))
- Fixing helmet theme bug. Fixes [#628](https://github.com/LemmyNet/lemmy-ui/issues/628) ([#629](https://github.com/LemmyNet/lemmy-ui/issues/629))
- Adding site ban from profile page. Fixes [#588](https://github.com/LemmyNet/lemmy-ui/issues/588) ([#627](https://github.com/LemmyNet/lemmy-ui/issues/627))
- Adding sidebar and subscribed collapse. Fixes [#609](https://github.com/LemmyNet/lemmy-ui/issues/609) ([#622](https://github.com/LemmyNet/lemmy-ui/issues/622))
- Adding a LEMMY_UI_DEBUG flag for eruda debugging ([#624](https://github.com/LemmyNet/lemmy-ui/issues/624))
- Adds OC ([#620](https://github.com/LemmyNet/lemmy-ui/issues/620))

# Lemmy v0.16.3 Release (2022-04-08)

## What is Lemmy?

Lemmy is a self-hosted social link aggregation and discussion platform. It is completely free and open, and not controlled by any company. This means that there is no advertising, tracking, or secret algorithms. Content is organized into communities, so it is easy to subscribe to topics that you are interested in, and ignore others. Voting is used to bring the most interesting items to the top.

## Major Changes

A full list of fixes is below, but this patch release includes federation compatibility and bug fixes, as well as fixing vulnerabilities in our websocket rate limiting.

## Upgrade notes

Besides the addition of a [search rate limit to the lemmy.hjson](https://github.com/LemmyNet/lemmy/blob/main/config/defaults.hjson#L39), there are no config or API changes.

Follow the [Docker or Ansible upgrade instructions here.](https://join-lemmy.org/docs/en/administration/administration.html)

## Support development

We (@dessalines and @nutomic) have been working full-time on Lemmy for almost two years. This is largely thanks to support from [NLnet foundation](https://nlnet.nl/). If you would like to support our efforts, please consider [donating](https://join-lemmy.org/donate).

If you'd like to support development, and make sure that we will always be available to work full time on Lemmy, consider [donating to support its development](https://join-lemmy.org/donate). We've spent hundreds of hours on Lemmy, and would like to be able to add more developers to our little open-source co-op as time goes on.

## Changes

### Lemmy Server

- Federate user account deletion (fixes [#1284](https://github.com/LemmyNet/lemmy/issues/1284)) ([#2199](https://github.com/LemmyNet/lemmy/issues/2199))
- Dont federate initial upvote ([#2196](https://github.com/LemmyNet/lemmy/issues/2196))
- Add missing mod log entries for federated actions (fixes [#1489](https://github.com/LemmyNet/lemmy/issues/1489)) ([#2198](https://github.com/LemmyNet/lemmy/issues/2198))
- Make sure application questionaire is required. Fixes [#2189](https://github.com/LemmyNet/lemmy/issues/2189)
- Fix verify_mod_action check for remote admin actions ([#2190](https://github.com/LemmyNet/lemmy/issues/2190))
- Run cargo upgrade ([#2176](https://github.com/LemmyNet/lemmy/issues/2176))
- Migrate towards using page.attachment field for url (ref [#2144](https://github.com/LemmyNet/lemmy/issues/2144)) ([#2182](https://github.com/LemmyNet/lemmy/issues/2182))
- Exclude removed/deleted posts from community outbox ([#2184](https://github.com/LemmyNet/lemmy/issues/2184))
- Fetch community outbox in parallel (fixes [#2180](https://github.com/LemmyNet/lemmy/issues/2180)) ([#2181](https://github.com/LemmyNet/lemmy/issues/2181))
- Adding a ban expires update job. Fixes [#2177](https://github.com/LemmyNet/lemmy/issues/2177)
- Add email translations ([#2175](https://github.com/LemmyNet/lemmy/issues/2175))
- Add test files for Friendica federation (fixes [#2144](https://github.com/LemmyNet/lemmy/issues/2144)) ([#2167](https://github.com/LemmyNet/lemmy/issues/2167))
- Lowering search rate limit. Fixes [#2153](https://github.com/LemmyNet/lemmy/issues/2153) ([#2154](https://github.com/LemmyNet/lemmy/issues/2154))
- Rate limit ws joins ([#2171](https://github.com/LemmyNet/lemmy/issues/2171))
- Delete unused diesel.toml file ([#2166](https://github.com/LemmyNet/lemmy/issues/2166))
- Rate limit websocket joins. ([#2165](https://github.com/LemmyNet/lemmy/issues/2165))
- Doing tests in sequential order. Fixes [#2158](https://github.com/LemmyNet/lemmy/issues/2158) ([#2163](https://github.com/LemmyNet/lemmy/issues/2163))
- Dont log errors when rate limit is hit (fixes [#2157](https://github.com/LemmyNet/lemmy/issues/2157)) ([#2161](https://github.com/LemmyNet/lemmy/issues/2161))
- Fix rate limit check for register. Fixes [#2159](https://github.com/LemmyNet/lemmy/issues/2159)
- GNU social compatibility ([#2100](https://github.com/LemmyNet/lemmy/issues/2100))
- Consolidate and lower reqwest timeouts. Fixes [#2150](https://github.com/LemmyNet/lemmy/issues/2150) ([#2151](https://github.com/LemmyNet/lemmy/issues/2151))
- Check that config is valid before saving ([#2152](https://github.com/LemmyNet/lemmy/issues/2152))
- Dont log error if duplicate activity is received (fixes [#2146](https://github.com/LemmyNet/lemmy/issues/2146)) ([#2148](https://github.com/LemmyNet/lemmy/issues/2148))
- WIP: Email localization (fixes [#500](https://github.com/LemmyNet/lemmy/issues/500)) ([#2053](https://github.com/LemmyNet/lemmy/issues/2053))
- If viewed actor isnt in db, fetch it from other instance ([#2145](https://github.com/LemmyNet/lemmy/issues/2145))
- Show rate limit algorithm. Fixes [#2136](https://github.com/LemmyNet/lemmy/issues/2136)
- Adjust retry interval for sending activities ([#2141](https://github.com/LemmyNet/lemmy/issues/2141))
- Add jerboa link to readme. Fixes [#2137](https://github.com/LemmyNet/lemmy/issues/2137)
- Forbid remote URLs for avatars/banners (fixes [#1618](https://github.com/LemmyNet/lemmy/issues/1618)) ([#2132](https://github.com/LemmyNet/lemmy/issues/2132))
- Remove docker/prod unused files (fixes [#2086](https://github.com/LemmyNet/lemmy/issues/2086)) ([#2133](https://github.com/LemmyNet/lemmy/issues/2133))
- Rework error handling (fixes [#1714](https://github.com/LemmyNet/lemmy/issues/1714)) ([#2135](https://github.com/LemmyNet/lemmy/issues/2135))
- Dont allow admin to add mod to remote community ([#2129](https://github.com/LemmyNet/lemmy/issues/2129))
- Reject federated downvotes if downvotes are disabled (fixes [#2124](https://github.com/LemmyNet/lemmy/issues/2124)) ([#2128](https://github.com/LemmyNet/lemmy/issues/2128))

### Lemmy UI

- Dont allow community urls like /c/{id} (fixes [#611](https://github.com/LemmyNet/lemmy-ui/issues/611)) ([#612](https://github.com/LemmyNet/lemmy-ui/issues/612))
- Fix loading indicator on search page (fixes [#443](https://github.com/LemmyNet/lemmy-ui/issues/443)) ([#606](https://github.com/LemmyNet/lemmy-ui/issues/606))
- Upgrade deps ([#604](https://github.com/LemmyNet/lemmy-ui/issues/604))
- Remove auth token from error message. Fixes [#600](https://github.com/LemmyNet/lemmy-ui/issues/600) ([#601](https://github.com/LemmyNet/lemmy-ui/issues/601))
- Fix error during new site setup ([#596](https://github.com/LemmyNet/lemmy-ui/issues/596))
- Differentiate between mods and admins in mod log ([#597](https://github.com/LemmyNet/lemmy-ui/issues/597))
- Fix comment fedilink (fixes [#594](https://github.com/LemmyNet/lemmy-ui/issues/594)) ([#595](https://github.com/LemmyNet/lemmy-ui/issues/595))

# Lemmy v0.16.1 Release

A few bug fixes:

## Lemmy

- Revert "Add logging to debug federation issues (ref [#2096](https://github.com/LemmyNet/lemmy/issues/2096)) ([#2099](https://github.com/LemmyNet/lemmy/issues/2099))" ([#2130](https://github.com/LemmyNet/lemmy/issues/2130))
- Dont allow admin to add mod to remote community ([#2129](https://github.com/LemmyNet/lemmy/issues/2129))
- Reject federated downvotes if downvotes are disabled (fixes [#2124](https://github.com/LemmyNet/lemmy/issues/2124)) ([#2128](https://github.com/LemmyNet/lemmy/issues/2128))

## Lemmy-ui

- Fix error during new site setup ([#596](https://github.com/LemmyNet/lemmy-ui/issues/596))
- Differentiate between mods and admins in mod log ([#597](https://github.com/LemmyNet/lemmy-ui/issues/597))
- Fix comment fedilink (fixes [#594](https://github.com/LemmyNet/lemmy-ui/issues/594)) ([#595](https://github.com/LemmyNet/lemmy-ui/issues/595))

# Lemmy v0.16.0 Release: Theming and Federation improvements (2022-03-08)

## What is Lemmy?

Lemmy is a self-hosted social link aggregation and discussion platform. It is completely free and open, and not controlled by any company. This means that there is no advertising, tracking, or secret algorithms. Content is organized into communities, so it is easy to subscribe to topics that you are interested in, and ignore others. Voting is used to bring the most interesting items to the top.

## Major Changes

### Theming

Customizing Lemmy is now much easier than before. Instance admins can select a default instance theme under `/admin` which applies to all users who are not logged in, and those who haven't explicitly picked a theme.

It is also possible now to add custom themes to an instance, without having to recompile lemmy-ui. When running with Docker, make sure that [these lines](https://github.com/LemmyNet/lemmy-ansible/pull/24/files) are present in docker-compose.yml (Ansible will add them automatically if you updated the repo). Then put your .css file into `./volumes/lemmy-ui/extra_themes`. The new theme can then be selected by users, or set as instance default.

For native installation (without Docker), themes are loaded by lemmy-ui from `./extra_themes` folder. A different path can be specified with `LEMMY_UI_EXTRA_THEMES_FOLDER` environment variable.

For instructions how to create a new theme, have a look at the [documentation](https://join-lemmy.org/docs/en/client_development/theming.html).

### Federation

@nutomic made many changes to federation to increase compatibility with other software. Lemmy can now receive deletions from [Pleroma], comments from [Friendica] and communities from [lotide](https://sr.ht/~vpzom/lotide/). Other actions were already compatible before. Mastodon can now display communities even when a user with identical name exists (but the user can't be viewed in that case). There were no breaking changes necessary, so federation is fully compatible with 0.15. If you notice something in another project that doesn't federate but should, please open an issue.

Multiple users have pointed out that posts, comments and votes don't federate reliably. We first attempted to fix this in [Lemmy 0.15.4](https://lemmy.ml/post/184152) a few days ago, but that didn't help much. Later @nutomic noticed that Lemmy was only sending out activities with 4 worker threads, which is not enough for a big instance like lemmy.ml. At the same time, many of those workers were taken up by sending to broken instances, trying to connect for a minute or more. This version adds a timeout and increases the number of workers.

### Federated bans

Until now, only community bans were federated, and the "Remove content" option didn't work over federation. The new version fixes this behaviour, so that both site bans and community bans federate, including "Remove content" option and expiry. Note that this change only affects new bans, those which were issued before upgrading to 0.16 will not be federated.

### Hide communities

@dayinjing implemented a funcionality for instance admins to hide controversial communities. A hidden community is only visible to those users who subscribe to it. This represents a milder alternative to removing a community. This functionality is not implemented in lemmy-ui yet, but admins can hide a community like this via command line:

```
curl -X PUT https://example.com/api/v3/community/hide \
    -H "Content-Type: application/json" \
    -d \
    '{"community_id":3,"hidden":true,"reason":"*reason for mod log*","auth":"*admin jwt token*"}'
```

### Jerboa: a new android app

To help adoption, and since most people use social media through their smartphones nowadays, @dessalines has been working on a native android app for Lemmy called [Jerboa](https://github.com/dessalines/jerboa), which is now on [F-Droid](https://f-droid.org/packages/com.jerboa) and [Google Play](https://play.google.com/store/apps/details?id=com.jerboa).

It is still at an alpha level, but is very usable. We'd love to have experienced android developers contribute to it.

This now makes three smartphone apps for Lemmy: [Lemmur and Jerboa for Android, and Remmel for iOS](https://join-lemmy.org/apps).

## Upgrade notes

Follow the [Docker or Ansible upgrade instructions here.](https://join-lemmy.org/docs/en/administration/administration.html)

There are three lemmy.hjson config changes. See [defaults.hjson](https://github.com/LemmyNet/lemmy/blob/main/config/defaults.hjson) for comments and default values.

- changed boolean `email.use_tls` to `email.tls_type`
- added `setup.default_theme`
- added `federation.worker_count`

## Support development

We (@dessalines and @nutomic) have been working full-time on Lemmy for almost two years. This is largely thanks to support from [NLnet foundation](https://nlnet.nl/). If you would like to support our efforts, please consider [donating](https://join-lemmy.org/donate).

If you'd like to support development, and make sure that we will always be available to work full time on Lemmy, consider [donating to support its development](https://join-lemmy.org/donate). We've spent hundreds of hours on Lemmy, and would like to be able to add more developers to our little open-source co-op as time goes on.

## Changes

### API

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.15.0 -> 0.16.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.15.0-rc.34...0.16.0-rc.1) .

### Config

- The config changes are [here.](https://github.com/LemmyNet/lemmy/compare/0.15.2...main#diff-bcc84ad7bb4d0687c679cb6b3711052d8eba74a8188578c7516a8fdb5584d01a)

### Lemmy Server

- Make delete activities backwards compatible with 0.15 ([#2114](https://github.com/LemmyNet/lemmy/issues/2114))
- Make activity queue worker count configurable, log stats ([#2113](https://github.com/LemmyNet/lemmy/issues/2113))
- Add timeout for sending activities ([#2112](https://github.com/LemmyNet/lemmy/issues/2112))
- Update `actix-*` dependencies to stable v4.
- Show nsfw communities if you are logged in and searching communities ([#2105](https://github.com/LemmyNet/lemmy/issues/2105))
- Fix resending activities (fixes [#1282](https://github.com/LemmyNet/lemmy/issues/1282)) ([#2109](https://github.com/LemmyNet/lemmy/issues/2109))
- Dont hardcode site id in Site::update ([#2110](https://github.com/LemmyNet/lemmy/issues/2110))
- send plain-text in email along with html ([#2107](https://github.com/LemmyNet/lemmy/issues/2107))
- Add site option for default theme ([#2104](https://github.com/LemmyNet/lemmy/issues/2104))
- Hide community v2 ([#2055](https://github.com/LemmyNet/lemmy/issues/2055))
- Reorganize federation tests ([#2092](https://github.com/LemmyNet/lemmy/issues/2092))
- Add logging to debug federation issues (ref [#2096](https://github.com/LemmyNet/lemmy/issues/2096)) ([#2099](https://github.com/LemmyNet/lemmy/issues/2099))
- Adding a reqwest timeout. Fixes [#2089](https://github.com/LemmyNet/lemmy/issues/2089) ([#2097](https://github.com/LemmyNet/lemmy/issues/2097))
- Upgrade to Rust 2021 edition ([#2093](https://github.com/LemmyNet/lemmy/issues/2093))
- Merge different delete activities for better compatibility (fixes [#2066](https://github.com/LemmyNet/lemmy/issues/2066)) ([#2073](https://github.com/LemmyNet/lemmy/issues/2073))
- Implement instance actor ([#1798](https://github.com/LemmyNet/lemmy/issues/1798))
- Use doku(skip) for opentelemetry_url config value (ref [#2085](https://github.com/LemmyNet/lemmy/issues/2085)) ([#2091](https://github.com/LemmyNet/lemmy/issues/2091))
- Alpha-ordering community follows. Fixes [#2062](https://github.com/LemmyNet/lemmy/issues/2062) ([#2079](https://github.com/LemmyNet/lemmy/issues/2079))
- Add federation tests for Friendica, improve parsing of source field (fixes [#2057](https://github.com/LemmyNet/lemmy/issues/2057)) ([#2070](https://github.com/LemmyNet/lemmy/issues/2070))

### Lemmy UI

- Rename theme files from _.min.css to _.css ([#590](https://github.com/LemmyNet/lemmy-ui/issues/590))
- Custom themes ([#584](https://github.com/LemmyNet/lemmy-ui/issues/584))
- Add option to set site default theme (fixes [#559](https://github.com/LemmyNet/lemmy-ui/issues/559))
- Adding nofollow to links. Fixes [#542](https://github.com/LemmyNet/lemmy-ui/issues/542) ([#543](https://github.com/LemmyNet/lemmy-ui/issues/543))
- Fix language names ([#580](https://github.com/LemmyNet/lemmy-ui/issues/580))
- Move fedi link in post listing location. Fixes [#569](https://github.com/LemmyNet/lemmy-ui/issues/569) ([#583](https://github.com/LemmyNet/lemmy-ui/issues/583))
- Don't redirect on server error. Fixes [#570](https://github.com/LemmyNet/lemmy-ui/issues/570) ([#582](https://github.com/LemmyNet/lemmy-ui/issues/582))
- Smart select inner content after bold or italics. Fixes [#497](https://github.com/LemmyNet/lemmy-ui/issues/497) ([#577](https://github.com/LemmyNet/lemmy-ui/issues/577))
- Fix comment jumping. Fixes [#529](https://github.com/LemmyNet/lemmy-ui/issues/529) ([#576](https://github.com/LemmyNet/lemmy-ui/issues/576))
- Add federated post and comment links. Fixes [#569](https://github.com/LemmyNet/lemmy-ui/issues/569) ([#575](https://github.com/LemmyNet/lemmy-ui/issues/575))
- Fix community comments iso fetch. Fixes [#572](https://github.com/LemmyNet/lemmy-ui/issues/572) ([#574](https://github.com/LemmyNet/lemmy-ui/issues/574))
- Don't allow transfer site. ([#551](https://github.com/LemmyNet/lemmy-ui/issues/551))
- Fix report page bugs. Fixes [#558](https://github.com/LemmyNet/lemmy-ui/issues/558) ([#568](https://github.com/LemmyNet/lemmy-ui/issues/568))
- Fix post title link bug. Fixes [#547](https://github.com/LemmyNet/lemmy-ui/issues/547) ([#563](https://github.com/LemmyNet/lemmy-ui/issues/563))
- Add markdown footnotes. Fixes [#561](https://github.com/LemmyNet/lemmy-ui/issues/561) ([#562](https://github.com/LemmyNet/lemmy-ui/issues/562))

# Lemmy v0.15.2 Release (2022-01-27)

A few bug fixes:

- Dont make webfinger request when viewing community/user profile (fixes [#1896](https://github.com/LemmyNet/lemmy/issues/1896)) ([#2049](https://github.com/LemmyNet/lemmy/issues/2049))
- Case-insensitive username at login ([#2010](https://github.com/LemmyNet/lemmy/issues/2010))
- Put community last in webfinger response (fixes [#2037](https://github.com/LemmyNet/lemmy/issues/2037)) ([#2047](https://github.com/LemmyNet/lemmy/issues/2047))
- Dont check for ban in MarkCommentAsRead (fixes [#2045](https://github.com/LemmyNet/lemmy/issues/2045)) ([#2054](https://github.com/LemmyNet/lemmy/issues/2054))
- Empty post bodies ([#2050](https://github.com/LemmyNet/lemmy/issues/2050))
- Add tombstone tests, better test errors ([#2046](https://github.com/LemmyNet/lemmy/issues/2046))
- Accept single object as to for arrays too ([#2048](https://github.com/LemmyNet/lemmy/issues/2048))
- Cleaning optional post bodies. Fixes [#2039](https://github.com/LemmyNet/lemmy/issues/2039) ([#2043](https://github.com/LemmyNet/lemmy/issues/2043))
- Fixing liking comment on blocked person. Fixes [#2033](https://github.com/LemmyNet/lemmy/issues/2033) ([#2042](https://github.com/LemmyNet/lemmy/issues/2042))
- Add tests for lotide federation, make lotide groups fetchable ([#2035](https://github.com/LemmyNet/lemmy/issues/2035))
- Remove unneeded dependency on activitystreams ([#2034](https://github.com/LemmyNet/lemmy/issues/2034))
- Fixing private instance check. Fixes [#2064](https://github.com/LemmyNet/lemmy/issues/2064) ([#2065](https://github.com/LemmyNet/lemmy/issues/2065))

# Lemmy v0.15.1 Release (2022-01-12)

Lemmy now has private instances, optional registration applications, optional email verification, and temporary bans! These are described in detail below.

Special thanks to @asonix for adding [tokio-console](https://github.com/LemmyNet/Lemmy/issues/2003) and [Jaeger + opentelemetry](https://github.com/LemmyNet/Lemmy/issues/1992) to our dev setups, so we can better identify performance bottlenecks.

## What is Lemmy?

[Lemmy](https://join-lemmy.org/) is similar to sites like Reddit, Lobste.rs, or Hacker News: you subscribe to communities you're interested in, post links and discussions, then vote and comment on them. Lemmy isn't just a reddit alternative; its a network of interconnected communities ran by different people and organizations, all combining to create a single, personalized front page of your favorite news, articles, and memes.

## Major Changes

### Required email verification

Admins can turn this on, and new users will need to verify their emails. Current users will not have to do this.

### Registration applications

Admins can now optionally make new users fill out an application to join your server. There is a new panel in their top bar where they can approve or deny pending applications.

This works in conjunction with the _require_email_ field. If that is also turned on, the application will only be shown after their email has been verified. The user will receive an email when they have been accepted.

### Closed / Private instances

The instance settings now includes a _private instance_ option, which if turned on, will only let logged in users view your site. Private instances was one of our first issues, and it was a large effort, so its great to finally have this completed.

### Temporary Bans

When banning users from your site or community, moderators can now optionally give a number of days for the ban to last.

### Allow comment replies from blocked users

It used to be that if a user blocked you, you couldn't respond to their public posts and comments. This is now fixed. They won't see your content, but others can.

## Upgrade notes

Follow the [Docker or Ansible upgrade instructions here.](https://join-lemmy.org/docs/en/administration/administration.html)

## Support development

If you'd like to support development, and make sure that we will always be available to work full time on Lemmy, consider [donating to support its development](https://join-lemmy.org/donate). We've spent hundreds of hours on Lemmy, and would like to be able to add more developers to our little open-source co-op as time goes on.

## Changes

### API

We've removed a list of banned users from `GetSite`, added a few endpoints related to registration applications, made a few changes allowing temporary bans, site settings, made a few changes to the login response. These are non-destructive and current clients should work with this release.

- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.14.0 -> 0.15.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.14.0-rc.1...0.15.0-rc.34) .

### Config

There is a new rate limit for creating new comments in the [config.hjson](https://github.com/LemmyNet/lemmy/blob/main/config/defaults.hjson#L36).

### Lemmy Server

- Adding temporary bans. Fixes [#1423](https://github.com/LemmyNet/Lemmy/issues/1423) ([#1999](https://github.com/LemmyNet/Lemmy/issues/1999))
- Add console-subscriber ([#2003](https://github.com/LemmyNet/Lemmy/issues/2003))
- Opentelemetry ([#1992](https://github.com/LemmyNet/Lemmy/issues/1992))
- Use correct encoding when fetching non-UTF-8 site metadata ([#2015](https://github.com/LemmyNet/Lemmy/issues/2015))
- Adding a banned endpoint for admins. Removing it from GetSite. Fixes [#1806](https://github.com/LemmyNet/Lemmy/issues/1806)
- Prevent panic on InboxRequestGuard
- Case-insensitive webfinger response. Fixes [#1955](https://github.com/LemmyNet/Lemmy/issues/1955) & [#1986](https://github.com/LemmyNet/Lemmy/issues/1986) ([#2005](https://github.com/LemmyNet/Lemmy/issues/2005))
- First pass at invite-only migration. ([#1949](https://github.com/LemmyNet/Lemmy/issues/1949))
- Upgrading pictrs. ([#1996](https://github.com/LemmyNet/Lemmy/issues/1996))
- Trying out an upgraded version of html5ever. [#1964](https://github.com/LemmyNet/Lemmy/issues/1964) ([#1991](https://github.com/LemmyNet/Lemmy/issues/1991))
- Adding min setup password length to the docs. Fixes [#1989](https://github.com/LemmyNet/Lemmy/issues/1989) ([#1990](https://github.com/LemmyNet/Lemmy/issues/1990))
- Test pleroma follow ([#1988](https://github.com/LemmyNet/Lemmy/issues/1988))
- Remove awc ([#1979](https://github.com/LemmyNet/Lemmy/issues/1979))
- Consolidate reqwest clients, use reqwest-middleware for tracing
- Don't drop error context when adding a message to errors ([#1958](https://github.com/LemmyNet/Lemmy/issues/1958))
- Change lemmur repo links ([#1977](https://github.com/LemmyNet/Lemmy/issues/1977))
- added deps - git and ca-certificates (for federation to work) and changed adduser to useradd so that user can be added non-interactively ([#1976](https://github.com/LemmyNet/Lemmy/issues/1976))
- Allow comment replies from blocked users. Fixes [#1793](https://github.com/LemmyNet/Lemmy/issues/1793) ([#1969](https://github.com/LemmyNet/Lemmy/issues/1969))
- Fix retry infinite loops. Fixes [#1964](https://github.com/LemmyNet/Lemmy/issues/1964) ([#1967](https://github.com/LemmyNet/Lemmy/issues/1967))
- Add lotide activities to tests
- Allow single item for to, cc, and @context
- Adding a captcha rate limit. Fixes [#1755](https://github.com/LemmyNet/Lemmy/issues/1755) ([#1941](https://github.com/LemmyNet/Lemmy/issues/1941))
- Dont send email notifications for edited comments (fixes [#1925](https://github.com/LemmyNet/Lemmy/issues/1925))
- Fix API dupes query. [#1878](https://github.com/LemmyNet/Lemmy/issues/1878)
- Fixing duped report view for admins. Fixes [#1933](https://github.com/LemmyNet/Lemmy/issues/1933) ([#1945](https://github.com/LemmyNet/Lemmy/issues/1945))
- Adding a GetComment endpoint. Fixes [#1919](https://github.com/LemmyNet/Lemmy/issues/1919) ([#1944](https://github.com/LemmyNet/Lemmy/issues/1944))
- Fix min title char count for post titles. Fixes [#1854](https://github.com/LemmyNet/Lemmy/issues/1854) ([#1940](https://github.com/LemmyNet/Lemmy/issues/1940))
- Adding MarkPostAsRead to API. Fixes [#1784](https://github.com/LemmyNet/Lemmy/issues/1784) ([#1946](https://github.com/LemmyNet/Lemmy/issues/1946))
- background-jobs 0.11 ([#1943](https://github.com/LemmyNet/Lemmy/issues/1943))
- Add tracing ([#1942](https://github.com/LemmyNet/Lemmy/issues/1942))
- Remove pointless community follower sort. ([#1939](https://github.com/LemmyNet/Lemmy/issues/1939))
- Use once_cell instead of lazy_static
- Adding unique constraint for activity ap_id. Fixes [#1878](https://github.com/LemmyNet/Lemmy/issues/1878) ([#1935](https://github.com/LemmyNet/Lemmy/issues/1935))
- Making public key required. Fixes [#1934](https://github.com/LemmyNet/Lemmy/issues/1934)
- Change NodeInfo `links` to an array
- Fixing fuzzy_search to escape like chars.
- Fix build error in [#1914](https://github.com/LemmyNet/Lemmy/issues/1914)
- Fix login ilike bug. Fixes [#1920](https://github.com/LemmyNet/Lemmy/issues/1920)
- Fix Smithereen webfinger, remove duplicate webfinger impl (fixes [#1916](https://github.com/LemmyNet/Lemmy/issues/1916))
- Dont announce comments, edited posts to Pleroma/Mastodon followers
- Community outbox should only contain activities sent by community (fixes [#1916](https://github.com/LemmyNet/Lemmy/issues/1916))
- Remove HTTP signature compatibility mode (its not necessary)
- Implement rate limits on comments

### Lemmy UI

- Fixed an issue with post embeds not being pushed to a new line [#544](https://github.com/LemmyNet/lemmy-ui/issues/544)
- Adding as and lt languages, Updating translations.
- Temp bans ([#524](https://github.com/LemmyNet/lemmy-ui/issues/524))
- Fix banner. Fixes [#466](https://github.com/LemmyNet/lemmy-ui/issues/466) ([#534](https://github.com/LemmyNet/lemmy-ui/issues/534))
- Making the modlog badge stand out more. Fixes [#531](https://github.com/LemmyNet/lemmy-ui/issues/531) ([#539](https://github.com/LemmyNet/lemmy-ui/issues/539))
- Add some fallback properties for display in older browsers ([#535](https://github.com/LemmyNet/lemmy-ui/issues/535))
- Private instances ([#523](https://github.com/LemmyNet/lemmy-ui/issues/523))
- Add nord theme. Fixes [#520](https://github.com/LemmyNet/lemmy-ui/issues/520) ([#527](https://github.com/LemmyNet/lemmy-ui/issues/527))
- Dont receive post room comments from blocked users. ([#516](https://github.com/LemmyNet/lemmy-ui/issues/516))
- Using console.error for error logs. ([#517](https://github.com/LemmyNet/lemmy-ui/issues/517))
- Fix issue with websocket buffer.
- Switching to websocket-ts. [#247](https://github.com/LemmyNet/lemmy-ui/issues/247) ([#515](https://github.com/LemmyNet/lemmy-ui/issues/515))
- Fix native language issue. (zh_Hant) ([#513](https://github.com/LemmyNet/lemmy-ui/issues/513))
- Fix tippy on component mount. Fixes [#509](https://github.com/LemmyNet/lemmy-ui/issues/509) ([#511](https://github.com/LemmyNet/lemmy-ui/issues/511))
- Fix docker latest ([#510](https://github.com/LemmyNet/lemmy-ui/issues/510))
- Enabling html tags in markdown. Fixes [#498](https://github.com/LemmyNet/lemmy-ui/issues/498)
- Fix comment scroll bug. Fixes [#492](https://github.com/LemmyNet/lemmy-ui/issues/492)
- Fixing error for null person_block. Fixes [#491](https://github.com/LemmyNet/lemmy-ui/issues/491)
- Trying to catch promise and json parse errors. [#489](https://github.com/LemmyNet/lemmy-ui/issues/489) ([#490](https://github.com/LemmyNet/lemmy-ui/issues/490))

# Lemmy Release v0.14.0: Federation with Mastodon and Pleroma (2021-11-17)

Today is an exciting day for the Lemmy project.

Almost one year after [first enabling federation](https://lemmy.ml/post/42833), we now federate with other projects for the first time! According to some people's definition, this finally makes us part of the Fediverse.

It took a lot of work to make this possible, so big thanks to [NLnet](https://nlnet.nl/) for funding our full time work on Lemmy, and to [@lanodan](https://queer.hacktivis.me/users/lanodan) and [@asonix](https://masto.asonix.dog/@asonix) for helping to figure out how Pleroma and Mastodon federation works (it's difficult because they have almost no documentation).

## Major Changes

### Federation code rewrite

The rewrite of the federation code started by @nutomic in August is now mostly complete. As a result, the code is much cleaner, and has tests to guarantee no breaking changes between Lemmy versions. As a side effect of this rewrite, it was now relatively easy to enable federation with other projects.

Mastodon and Pleroma users can:

- View Lemmy communities, user profiles, posts and comments
- Follow Lemmy communities to receive new posts and comments
- Replies (mentions) work in both directions, including notifications

In addition, Pleroma users can exchange private messages with Lemmy users.

Note that Pleroma and Mastodon rely on a compatibility mode in Lemmy, which means that they won't receive events like Deletes or Votes. Other projects whose federation works similar to Pleroma/Mastodon will likely also federate.

### Hardcoded slur filter removed

Lemmy finally has essential moderation tools (reporting, user/community blocking), so the hardcoded filter isn't necessary anymore. If you want to keep using the slur filter, copy [these lines](https://github.com/LemmyNet/lemmy/blob/b18ea3e0cc620c3f97f9804c09b92f193809b846/config/config.hjson#L8-L12) to your config file when upgrading, and adjust to your liking.

## Upgrade notes

Federation with Pleroma/Mastodon works automatically, you don't need to change anything, assuming that your allowlist/blocklist configuration permits it.

Note that Mastodon and Pleroma are much, much bigger than Lemmy at this point, with a combined 3 milion users and 4500 instances, compared to 20.000 users and 35 instances for Lemmy ([source](https://the-federation.info/)). The existing mod tools in Lemmy might not be adequate to handle that at the moment.

Be aware that if you have federation enabled in the Lemmy config, Mastodon and Pleroma users can now fetch all posts and comments, to view them and share with their followers. The Lemmy blocklist/allowlist can not prevent this, it only prevents posts/comments from blocked instances to be shown on your own instance. The only solution to this problem is disabling federation, or waiting for [signed fetch](https://github.com/LemmyNet/lemmy/issues/868) to be implemented.

If you want to use federation, but review new instances before federating with them, use the allowlist. You can switch from open federation to allowlist federation by pasting the output of the command below into `federation.allowed_instances` in the Lemmy config.

```
curl https://your-instance.com/api/v3/site | jq -c .federated_instances.linked
```

The [`lemmy.hjson` `additional_slurs` field has changed its name to `slur_filter`. ](https://github.com/LemmyNet/lemmy/blob/b18ea3e0cc620c3f97f9804c09b92f193809b846/config/config.hjson#L8-L12)

Follow the [Docker or Ansible upgrade instructions here.](https://join-lemmy.org/docs/en/administration/administration.html)

## Lemmy-Ansible

We've now separated our ansible install method (the preferred way to deploy Lemmy) into its own repo, [lemmy-ansible](https://github.com/LemmyNet/lemmy-ansible). Let us know if you need help migrating existing installations over to it.

## Changes

### API

- There is now a `GetUnreadCount` in the API to check the count of your unread messages, replies, and mentions.
- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.13.0 -> 0.14.0-rc.1](https://github.com/LemmyNet/lemmy-js-client/compare/0.13.0...0.14.0-rc.1) .

### Lemmy Server

- More federation compat ([#1894](https://github.com/LemmyNet/Lemmy/issues/1894))
- Adding clippy:unwrap to husky. Fixes [#1892](https://github.com/LemmyNet/Lemmy/issues/1892) ([#1893](https://github.com/LemmyNet/Lemmy/issues/1893))
- Remove header guard for activitypub routes
- Add federation test cases for Smithereen and Mastodon
- Reduce stack memory usage in apub code
- Remove ActivityFields trait, deserialize into another struct instead
- Check if post or comment are deleted first. Fixes [#1864](https://github.com/LemmyNet/Lemmy/issues/1864) ([#1867](https://github.com/LemmyNet/Lemmy/issues/1867))
- Correctly use and document check_is_apub_id_valid() param use_strict_allowlist
- Convert note.content and chat_message.content to html (fixes [#1871](https://github.com/LemmyNet/Lemmy/issues/1871))
- Upgrade background_jobs to 0.9.1 [#1820](https://github.com/LemmyNet/Lemmy/issues/1820) ([#1875](https://github.com/LemmyNet/Lemmy/issues/1875))
- Fix husky fmt hook. ([#1868](https://github.com/LemmyNet/Lemmy/issues/1868))
- Renaming to slur_filter. Fixes [#1773](https://github.com/LemmyNet/Lemmy/issues/1773) ([#1801](https://github.com/LemmyNet/Lemmy/issues/1801))
- Three instance inbox bug ([#1866](https://github.com/LemmyNet/Lemmy/issues/1866))
- Remove ansible from this repo. ([#1829](https://github.com/LemmyNet/Lemmy/issues/1829))
- Rewrite collections to use new fetcher ([#1861](https://github.com/LemmyNet/Lemmy/issues/1861))
- Dont blank out post or community info. Fixes [#1813](https://github.com/LemmyNet/Lemmy/issues/1813) ([#1841](https://github.com/LemmyNet/Lemmy/issues/1841))
- Format config/defaults.hjson before committing ([#1860](https://github.com/LemmyNet/Lemmy/issues/1860))
- Breaking apub changes ([#1859](https://github.com/LemmyNet/Lemmy/issues/1859))
- Pleroma federation2 ([#1855](https://github.com/LemmyNet/Lemmy/issues/1855))
- Create a custom pre-commit hook, generates config/defaults.hjson ([#1857](https://github.com/LemmyNet/Lemmy/issues/1857))
- Add cargo metadata to all crates ([#1853](https://github.com/LemmyNet/Lemmy/issues/1853))
- Add both (De)Serialize to all models ([#1851](https://github.com/LemmyNet/Lemmy/issues/1851))
- Adding GetUnreadCount to the API. Fixes [#1794](https://github.com/LemmyNet/Lemmy/issues/1794) ([#1842](https://github.com/LemmyNet/Lemmy/issues/1842))
- Federate reports ([#1830](https://github.com/LemmyNet/Lemmy/issues/1830))
- Fix saved posts and hide read posts issue. Fixes [#1839](https://github.com/LemmyNet/Lemmy/issues/1839) ([#1840](https://github.com/LemmyNet/Lemmy/issues/1840))
- Dont allow posts to deleted / removed communities. Fixes [#1827](https://github.com/LemmyNet/Lemmy/issues/1827) ([#1828](https://github.com/LemmyNet/Lemmy/issues/1828))
- Dont swallow API errors (fixes [#1834](https://github.com/LemmyNet/Lemmy/issues/1834)) ([#1837](https://github.com/LemmyNet/Lemmy/issues/1837))
- Fix federation of initial post/comment vote (fixes [#1824](https://github.com/LemmyNet/Lemmy/issues/1824)) ([#1835](https://github.com/LemmyNet/Lemmy/issues/1835))
- Fix clippy warnings added in nightly ([#1833](https://github.com/LemmyNet/Lemmy/issues/1833))
- Admins can view all reports. Fixes [#1810](https://github.com/LemmyNet/Lemmy/issues/1810) ([#1825](https://github.com/LemmyNet/Lemmy/issues/1825))
- Adding a message_id to emails. Fixes [#1807](https://github.com/LemmyNet/Lemmy/issues/1807) ([#1826](https://github.com/LemmyNet/Lemmy/issues/1826))
- Generate config docs from code ([#1786](https://github.com/LemmyNet/Lemmy/issues/1786))
- Trying a background_jobs fix. [#1820](https://github.com/LemmyNet/Lemmy/issues/1820) ([#1822](https://github.com/LemmyNet/Lemmy/issues/1822))
- mark parent as read on reply ([#1819](https://github.com/LemmyNet/Lemmy/issues/1819))
- Move code to apub library ([#1795](https://github.com/LemmyNet/Lemmy/issues/1795))
- Adding honeypot to user and post creation. Fixes [#1802](https://github.com/LemmyNet/Lemmy/issues/1802) ([#1803](https://github.com/LemmyNet/Lemmy/issues/1803))
- Add database host back into config file ([#1805](https://github.com/LemmyNet/Lemmy/issues/1805))

### Lemmy UI

- Updating translations.
- Fixing unload ([#487](https://github.com/LemmyNet/lemmy-ui/issues/487))
- Fix setup password. Fixes [#478](https://github.com/LemmyNet/lemmy-ui/issues/478) ([#484](https://github.com/LemmyNet/lemmy-ui/issues/484))
- Adding post comment scrolling hack. Fixes [#480](https://github.com/LemmyNet/lemmy-ui/issues/480) [#486](https://github.com/LemmyNet/lemmy-ui/issues/486)
- Navbar links ([#476](https://github.com/LemmyNet/lemmy-ui/issues/476))
- Try fixing crypto node bug. Fixes [#473](https://github.com/LemmyNet/lemmy-ui/issues/473) ([#474](https://github.com/LemmyNet/lemmy-ui/issues/474))
- Use community title and user display name for dropdown.
- Mahanstreamer userpage ([#471](https://github.com/LemmyNet/lemmy-ui/issues/471))
- Using i18next compatibility v3 ([#465](https://github.com/LemmyNet/lemmy-ui/issues/465))
- Show original created time tooltip ([#462](https://github.com/LemmyNet/lemmy-ui/issues/462))
- Revert version of i18next to fix plurals. Fixes [#451](https://github.com/LemmyNet/lemmy-ui/issues/451) ([#460](https://github.com/LemmyNet/lemmy-ui/issues/460))
- Fixing cross-posts showing on initial load. Fixes [#457](https://github.com/LemmyNet/lemmy-ui/issues/457) ([#464](https://github.com/LemmyNet/lemmy-ui/issues/464))
- Show bot account info. Fixes [#458](https://github.com/LemmyNet/lemmy-ui/issues/458) ([#463](https://github.com/LemmyNet/lemmy-ui/issues/463))
- Very weak password check ([#461](https://github.com/LemmyNet/lemmy-ui/issues/461))
- Simplifying getunreadcount. ([#455](https://github.com/LemmyNet/lemmy-ui/issues/455))
- ui changes for marking comment as read on reply ([#454](https://github.com/LemmyNet/lemmy-ui/issues/454))
- hide mod actions appropriately fix [#441](https://github.com/LemmyNet/lemmy-ui/issues/441) ([#447](https://github.com/LemmyNet/lemmy-ui/issues/447))
- Add honeypot for user and form creation. Fixes [#433](https://github.com/LemmyNet/lemmy-ui/issues/433) ([#435](https://github.com/LemmyNet/lemmy-ui/issues/435))

# Lemmy v0.13.3 Release (2021-10-13)

- Dont swallow API errors (fixes [#1834](https://github.com/LemmyNet/lemmy/issues/1834)) ([#1837](https://github.com/LemmyNet/lemmy/issues/1837))
- Fix clippy warnings added in nightly ([#1833](https://github.com/LemmyNet/lemmy/issues/1833))
- Fix federation of initial post/comment vote (fixes [#1824](https://github.com/LemmyNet/lemmy/issues/1824)) ([#1835](https://github.com/LemmyNet/lemmy/issues/1835))
- Trying a background_jobs fix. [#1820](https://github.com/LemmyNet/lemmy/issues/1820)

# Lemmy v0.13.0 Release (2021-09-30)

Since our last release earlier this month, we've had [~30](https://github.com/LemmyNet/lemmy/compare/0.12.0...main) commits to Lemmy.

## Major Changes

- Added comment and post reporting in the front end, and cleaned up the reporting API.
  - _Note: these are local-only currently, reports are not yet federated._
- The JWT secret is now auto-generated by the database.
  - _Note: this will log out all users, so users will have to log in again._
- Lots of smaller UI fixes listed below.

## Upgrade notes

### Servers

If you'd like to make a DB backup before upgrading, follow [this guide](https://join.lemmy.ml/docs/en/administration/backup_and_restore.html).

To upgrade your instance to `v0.13.0`, simply follow the instructions in the documentation:

- [Upgrade with manual Docker installation](https://join.lemmy.ml/docs/en/administration/install_docker.html#updating)
- [Upgrade with Ansible installation](https://join.lemmy.ml/docs/en/administration/install_ansible.html)

## Changes

### Lemmy Server

#### General

- Adding a user agent. Fixes [#1769](https://github.com/LemmyNet/lemmy/issues/1769)
- Ansible changes ([#1781](https://github.com/LemmyNet/lemmy/issues/1781))
- Clean up reporting ([#1776](https://github.com/LemmyNet/lemmy/issues/1776))
- Implement webmention support (fixes [#1395](https://github.com/LemmyNet/lemmy/issues/1395))
- Move jwt secret from config to database (fixes [#1728](https://github.com/LemmyNet/lemmy/issues/1728))
- Set a 10 char minimum password length.
- Dont pass accept-encoding header to pictrs (ref [#1734](https://github.com/LemmyNet/lemmy/issues/1734)) ([#1738](https://github.com/LemmyNet/lemmy/issues/1738))

#### API

- There are no breaking API changes, only the addition of reporting endpoints.
- A full list of the API changes can be seen on this diff of [lemmy-js-client: 0.12.0 -> 0.13.0](https://github.com/LemmyNet/lemmy-js-client/compare/0.12.0...0.13.0) .

#### Federation

- Rewrite fetcher ([#1792](https://github.com/LemmyNet/lemmy/issues/1792))

### Lemmy UI

- Adding bn, ml, and cs langs
- Reporting ([#434](https://github.com/LemmyNet/lemmy-ui/issues/434))
- Splitting login and signup pages. Fixes [#386](https://github.com/LemmyNet/lemmy-ui/issues/386) ([#431](https://github.com/LemmyNet/lemmy-ui/issues/431))
- Fixing image in newtab. Fixes [#382](https://github.com/LemmyNet/lemmy-ui/issues/382) ([#430](https://github.com/LemmyNet/lemmy-ui/issues/430))
- Navigate away from login page if already logged in. ([#429](https://github.com/LemmyNet/lemmy-ui/issues/429))
- Add username validation message. Fixes [#387](https://github.com/LemmyNet/lemmy-ui/issues/387) ([#428](https://github.com/LemmyNet/lemmy-ui/issues/428))
- Password strength meter ([#427](https://github.com/LemmyNet/lemmy-ui/issues/427))
- Fix community display name overflow. Fixes [#390](https://github.com/LemmyNet/lemmy-ui/issues/390) ([#425](https://github.com/LemmyNet/lemmy-ui/issues/425))
- Fix logout bug. Fixes [#391](https://github.com/LemmyNet/lemmy-ui/issues/391) ([#424](https://github.com/LemmyNet/lemmy-ui/issues/424))
- Fix up post, profile and community forms. Fixes [#409](https://github.com/LemmyNet/lemmy-ui/issues/409) ([#423](https://github.com/LemmyNet/lemmy-ui/issues/423))
- Adding markdown audio and video embeds. Fixes [#420](https://github.com/LemmyNet/lemmy-ui/issues/420) ([#421](https://github.com/LemmyNet/lemmy-ui/issues/421))
- Adding Si simplifier ([#418](https://github.com/LemmyNet/lemmy-ui/issues/418))
- Fix profile paging. Fixes [#416](https://github.com/LemmyNet/lemmy-ui/issues/416) ([#417](https://github.com/LemmyNet/lemmy-ui/issues/417))
- Use my fork of inferno-i18next. Fixes [#413](https://github.com/LemmyNet/lemmy-ui/issues/413) ([#415](https://github.com/LemmyNet/lemmy-ui/issues/415))
- Add version to package.json . Fixes [#411](https://github.com/LemmyNet/lemmy-ui/issues/411)

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

_Note: Issue links are below._

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

|             | v0.9.0 (Rust 1.47) | v0.10.0 (Rust 1.47) | v0.10.0 (Rust 1.51) |
| ----------- | ------------------ | ------------------- | ------------------- |
| Clean       | 140s               | 146s                | 119s                |
| Incremental | 28s                | 22s                 | 19s                 |

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

We've also seen the first release of [Lemmur](https://github.com/LemmurOrg/lemmur/releases/tag/v0.1.1), an android / iOS (soon) / windows / linux client, as well as [Lemmer](https://github.com/uuttff8/Lemmy-iOS), a native iOS client. Much thanks to @krawieck, @shilangyu, and @uuttff8 for making these great clients. If you can, please contribute to their [patreon](https://www.patreon.com/lemmur) to help fund lemmur development.

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
