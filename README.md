<div align="center">

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/dessalines/lemmy.svg)
[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)
![GitHub stars](https://img.shields.io/github/stars/dessalines/lemmy?style=social)
</div>

<p align="center">
  <a href="https://dev.lemmy.ml/" rel="noopener">
 <img width=200px height=200px src="ui/assets/favicon.svg"></a>

 <h3 align="center"><a href="https://dev.lemmy.ml">Lemmy</a></h3>
  <p align="center">
    A link aggregator / reddit clone for the fediverse.
    <br />
    <br />
    <a href="https://dev.lemmy.ml">View Site</a>
    ·
    <a href="https://dev.lemmy.ml/docs/index.html">Documentation</a>
    ·
    <a href="https://github.com/dessalines/lemmy/issues">Report Bug</a>
    ·
    <a href="https://github.com/dessalines/lemmy/issues">Request Feature</a>
    ·
    <a href="https://github.com/dessalines/lemmy/blob/master/RELEASES.md">Releases</a>
  </p>
</p>

## About The Project

Front Page|Post
---|---
![main screen](https://i.imgur.com/kZSRcRu.png)|![chat screen](https://i.imgur.com/4XghNh6.png)

[Lemmy](https://github.com/dessalines/lemmy) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), [Raddle](https://raddle.me), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

The overall goal is to create an easily self-hostable, decentralized alternative to reddit and other link aggregators, outside of their corporate control and meddling.

Each lemmy server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

*Note: Federation is still in active development*

### Why's it called Lemmy?

- Lead singer from [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- The old school [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- The [Koopa from Super Mario](https://www.mariowiki.com/Lemmy_Koopa).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

### Built With

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## Features

- Open source, [AGPL License](/LICENSE).
- Self hostable, easy to deploy.
  - Comes with [Docker](#docker), [Ansible](#ansible), [Kubernetes](#kubernetes).
- Clean, mobile-friendly interface.
  - Only a minimum of a username and password is required to sign up!
  - User avatar support.
  - Live-updating Comment threads.
  - Full vote scores `(+/-)` like old reddit.
  - Themes, including light, dark, and solarized.
  - Emojis with autocomplete support. Start typing `:`
  - User tagging using `@`, Community tagging using `#`.
  - Integrated image uploading in both posts and comments.
  - A post can consist of a title and any combination of self text, a URL, or nothing else.
  - Notifications, on comment replies and when you're tagged.
    - Notifications can be sent via email.
  - i18n / internationalization support.
  - RSS / Atom feeds for `All`, `Subscribed`, `Inbox`, `User`, and `Community`.
- Cross-posting support.
  - A *similar post search* when creating new posts. Great for question / answer communities.
- Moderation abilities.
  - Public Moderation Logs.
  - Can sticky posts to the top of communities.
  - Both site admins, and community moderators, who can appoint other moderators.
  - Can lock, remove, and restore posts and comments.
  - Can ban and unban users from communities and the site.
  - Can transfer site and communities to others.
- Can fully erase your data, replacing all posts and comments.
- NSFW post / community support.
- OEmbed support via Iframely.
- High performance.
  - Server is written in rust.
  - Front end is `~80kB` gzipped.
  - Supports arm64 / Raspberry Pi.

## Installation

- [Docker](https://dev.lemmy.ml/docs/administration_install_docker.html)
- [Ansible](https://dev.lemmy.ml/docs/administration_install_ansible.html)
- [Kubernetes](https://dev.lemmy.ml/docs/administration_install_kubernetes.html)
## Support / Donate

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

- [Support on Liberapay.](https://liberapay.com/Lemmy)
- [Support on Patreon](https://www.patreon.com/dessalines).
- [List of Sponsors](https://dev.lemmy.ml/sponsors).

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Contributing

- [Contributing instructions](https://dev.lemmy.ml/docs/contributing.html)
- [Docker Development](https://dev.lemmy.ml/docs/contributing_docker_development.html)
- [Local Development](https://dev.lemmy.ml/docs/contributing_local_development.html)

### Translations 

If you'd like to add translations, take a look at the [English translation file](ui/src/translations/en.ts).

- Languages supported: Brazilian Portuguese (`pt-br`), Catalan, (`ca`), Farsi (`fa`), English (`en`), Chinese (`zh`), Dutch (`nl`), Esperanto (`eo`), Finnish (`fi`), French (`fr`), Spanish (`es`), Swedish (`sv`), German (`de`), Russian (`ru`), Italian (`it`).

<!-- translations -->

lang | done | missing
---- | ---- | -------
ca | 97% | cross_posted_to,old,support_on_liberapay,couldnt_get_comments,post_title_too_long,time,action
de | 86% | cross_posted_to,create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,old,docs,message_sent,messages,old_password,matrix_user_id,private_message_disclaimer,send_notifications_to_email,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,donate_to_lemmy,donate,from,logged_in,couldnt_get_comments,post_title_too_long,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
fa | 71% | cross_post,cross_posted_to,subscribed_to_communities,trending_communities,create_private_message,send_secure_message,send_message,message,mod,mods,moderates,remove_as_mod,appoint_as_mod,modlog,stickied,ban,ban_from_site,unban,unban_from_site,banned,number_of_subscribers,subscribers,both,saved,unsubscribe,subscribe,subscribed,old,api,docs,inbox,inbox_for,message_sent,notifications_error,messages,no_email_setup,matrix_user_id,private_message_disclaimer,url,body,copy_suggested_title,community,expand_here,subscribe_to_communities,theme,sponsor_message,support_on_liberapay,general_sponsors,joined,by,to,from,landing_0,logged_in,couldnt_get_comments,community_moderator_already_exists,community_follower_already_exists,community_user_already_banned,post_title_too_long,no_slurs,admin_already_created,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
eo | 73% | cross_posted_to,number_of_communities,create_private_message,send_secure_message,send_message,message,preview,upload_image,avatar,upload_avatar,show_avatars,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,old,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,theme,support_on_liberapay,donate_to_lemmy,donate,from,are_you_sure,yes,no,logged_in,couldnt_get_comments,post_title_too_long,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
es | 99% | cross_posted_to,couldnt_get_comments,post_title_too_long
fi | 97% | cross_posted_to,old,support_on_liberapay,couldnt_get_comments,post_title_too_long,time,action
fr | 100% | upload_avatar,show_avatars
it | 82% | cross_posted_to,create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,archive_link,old,docs,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,donate_to_lemmy,donate,from,logged_in,couldnt_get_comments,post_title_too_long,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
nl | 98% | cross_posted_to,couldnt_get_comments,post_title_too_long,time,action
pt-br | 99% | couldnt_get_comments,post_title_too_long
ru | 70% | cross_posts,cross_post,cross_posted_to,number_of_communities,create_private_message,send_secure_message,send_message,message,preview,upload_image,avatar,upload_avatar,show_avatars,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,old,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,recent_comments,theme,support_on_liberapay,donate_to_lemmy,donate,monero,by,to,from,transfer_community,transfer_site,are_you_sure,yes,no,logged_in,couldnt_get_comments,post_title_too_long,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
sv | 81% | cross_posted_to,create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,archive_link,old,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,support_on_liberapay,donate_to_lemmy,donate,from,logged_in,couldnt_get_comments,post_title_too_long,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
zh | 69% | cross_posts,cross_post,cross_posted_to,users,number_of_communities,create_private_message,send_secure_message,send_message,message,preview,upload_image,avatar,upload_avatar,show_avatars,formatting_help,view_source,sticky,unsticky,archive_link,settings,stickied,delete_account,delete_account_confirm,banned,creator,number_online,old,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,recent_comments,nsfw,show_nsfw,theme,donate_to_lemmy,donate,monero,by,to,from,transfer_community,transfer_site,are_you_sure,yes,no,logged_in,couldnt_get_comments,post_title_too_long,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message,time,action
<!-- translationsstop -->

If you'd like to update this report, run:

```bash 
cd ui
ts-node translation_report.ts
```

## Contact

- [Mastodon](https://mastodon.social/@LemmyDev) - [![Mastodon Follow](https://img.shields.io/mastodon/follow/810572?domain=https%3A%2F%2Fmastodon.social&style=social)](https://mastodon.social/@LemmyDev)
- [Matrix](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org) - [![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
- [GitHub](https://github.com/dessalines/lemmy)
- [Gitea](https://yerbamate.dev/dessalines/lemmy)
- [GitLab](https://gitlab.com/dessalines/lemmy)

## Credits

Logo made by Andy Cuccaro (@andycuccaro) under the CC-BY-SA 4.0 license.
