<p align="center">
  <a href="" rel="noopener">
 <img width=200px height=200px src="ui/assets/favicon.svg"></a>
</p>

<h3 align="center">Lemmy</h3>

<div align="center">

[![Github](https://img.shields.io/badge/-Github-blue)](https://github.com/dessalines/lemmy)
[![Gitlab](https://img.shields.io/badge/-Gitlab-yellowgreen)](https://gitlab.com/dessalines/lemmy)
![Mastodon Follow](https://img.shields.io/mastodon/follow/810572?domain=https%3A%2F%2Fmastodon.social&style=social)
![GitHub stars](https://img.shields.io/github/stars/dessalines/lemmy?style=social)
[![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/dessalines/lemmy.svg)
[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/dessalines/lemmy.svg)
![GitHub repo size](https://img.shields.io/github/repo-size/dessalines/lemmy.svg)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)
[![Patreon](https://img.shields.io/badge/-Support%20on%20Patreon-blueviolet.svg)](https://www.patreon.com/dessalines)
</div>

---

<p align="center">A link aggregator / reddit clone for the fediverse.
    <br> 
</p>

[Lemmy Dev instance](https://dev.lemmy.ml) *for testing purposes only*

This is a **very early beta version**, and a lot of features are currently broken or in active development, such as federation.

Front Page|Post
---|---
![main screen](https://i.imgur.com/kZSRcRu.png)|![chat screen](https://i.imgur.com/4XghNh6.png)

## 📝 Table of Contents

<!-- toc -->

- [Features](#features)
- [About](#about)
  * [Why's it called Lemmy?](#whys-it-called-lemmy)
- [Install](#install)
  * [Docker](#docker)
    + [Updating](#updating)
  * [Ansible](#ansible)
  * [Kubernetes](#kubernetes)
- [Develop](#develop)
  * [Docker Development](#docker-development)
  * [Local Development](#local-development)
    + [Requirements](#requirements)
    + [Set up Postgres DB](#set-up-postgres-db)
    + [Running](#running)
- [Documentation](#documentation)
- [Support](#support)
- [Translations](#translations)
- [Credits](#credits)

<!-- tocstop -->

## Features

- Open source, [AGPL License](/LICENSE).
- Self hostable, easy to deploy.
  - Comes with [Docker](#docker), [Ansible](#ansible), [Kubernetes](#kubernetes).
- Clean, mobile-friendly interface.
  - Live-updating Comment threads.
  - Full vote scores `(+/-)` like old reddit.
  - Themes, including light, dark, and solarized.
  - Emojis with autocomplete support. Start typing `:`
  - User tagging using `@`, Community tagging using `#`.
  - Notifications, on comment replies and when you're tagged.
  - i18n / internationalization support.
  - RSS / Atom feeds for `All`, `Subscribed`, `Inbox`, `User`, and `Community`.
- Cross-posting support.
  - A *similar post search* when creating new posts. Great for question / answer communities.
- Moderation abilities.
  - Public Moderation Logs.
  - Both site admins, and community moderators, who can appoint other moderators.
  - Can lock, remove, and restore posts and comments.
  - Can ban and unban users from communities and the site.
  - Can transfer site and communities to others.
- Can fully erase your data, replacing all posts and comments.
- NSFW post / community support.
- High performance.
  - Server is written in rust.
  - Front end is `~80kB` gzipped.
  - Supports arm64 / Raspberry Pi.

## About

[Lemmy](https://github.com/dessalines/lemmy) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), [Raddle](https://raddle.me), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

The overall goal is to create an easily self-hostable, decentralized alternative to reddit and other link aggregators, outside of their corporate control and meddling.

Each lemmy server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

### Why's it called Lemmy?

- Lead singer from [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- The old school [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- The [Koopa from Super Mario](https://www.mariowiki.com/Lemmy_Koopa).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://www.infernojs.org), [Typescript](https://www.typescriptlang.org/) and [Diesel](http://diesel.rs/).

## Install

### Docker

Make sure you have both docker and docker-compose(>=`1.24.0`) installed:

```bash
mkdir lemmy/
cd lemmy/
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/.env
# Edit the .env if you want custom passwords
docker-compose up -d
```

and go to http://localhost:8536.

[A sample nginx config](/ansible/templates/nginx.conf), could be setup with:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/ansible/templates/nginx.conf
# Replace the {{ vars }}
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
```
#### Updating

To update to the newest version, run:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
docker-compose up -d
```

### Ansible

First, you need to [install Ansible on your local computer](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html) (e.g. using `sudo apt install ansible`) or the equivalent for you platform.

Then run the following commands on your local computer:

```bash
git clone https://github.com/dessalines/lemmy.git
cd lemmy/ansible/
cp inventory.example inventory
nano inventory # enter your server, domain, contact email
ansible-playbook lemmy.yml --become
```

### Kubernetes

You'll need to have an existing Kubernetes cluster and [storage class](https://kubernetes.io/docs/concepts/storage/storage-classes/).
Setting this up will vary depending on your provider.
To try it locally, you can use [MicroK8s](https://microk8s.io/) or [Minikube](https://kubernetes.io/docs/tasks/tools/install-minikube/).

Once you have a working cluster, edit the environment variables and volume sizes in `docker/k8s/*.yml`.
You may also want to change the service types to use `LoadBalancer`s depending on where you're running your cluster (add `type: LoadBalancer` to `ports)`, or `NodePort`s.
By default they will use `ClusterIP`s, which will allow access only within the cluster. See the [docs](https://kubernetes.io/docs/concepts/services-networking/service/) for more on networking in Kubernetes.

**Important** Running a database in Kubernetes will work, but is generally not recommended.
If you're deploying on any of the common cloud providers, you should consider using their managed database service instead (RDS, Cloud SQL, Azure Databse, etc.).

Now you can deploy:

```bash
# Add `-n foo` if you want to deploy into a specific namespace `foo`;
# otherwise your resources will be created in the `default` namespace.
kubectl apply -f docker/k8s/db.yml
kubectl apply -f docker/k8s/pictshare.yml
kubectl apply -f docker/k8s/lemmy.yml
```

If you used a `LoadBalancer`, you should see it in your cloud provider's console.

## Develop

### Docker Development

Run:

```bash
git clone https://github.com/dessalines/lemmy
cd lemmy/docker/dev
./docker_update.sh # This builds and runs it, updating for your changes
```

and go to http://localhost:8536.

### Local Development

#### Requirements

- [Rust](https://www.rust-lang.org/)
- [Yarn](https://yarnpkg.com/en/)
- [Postgres](https://www.postgresql.org/)

#### Set up Postgres DB

```bash
 psql -c "create user lemmy with password 'password' superuser;" -U postgres
 psql -c 'create database lemmy with owner lemmy;' -U postgres
 export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
```

#### Running

```bash
git clone https://github.com/dessalines/lemmy
cd lemmy
./install.sh
# For live coding, where both the front and back end, automagically reload on any save, do:
# cd ui && yarn start
# cd server && cargo watch -x run
```

## Documentation

- [Websocket API for App developers](docs/api.md)
- [ActivityPub API.md](docs/apub_api_outline.md)
- [Goals](docs/goals.md)
- [Ranking Algorithm](docs/ranking.md)

## Support

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

- [Support on Patreon](https://www.patreon.com/dessalines).
- [Sponsor List](https://dev.lemmy.ml/sponsors).
- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Translations 

If you'd like to add translations, take a look a look at the [English translation file](ui/src/translations/en.ts).

- Languages supported: English (`en`), Chinese (`zh`), Dutch (`nl`), Esperanto (`eo`), French (`fr`), Spanish (`es`), Swedish (`sv`), German (`de`), Russian (`ru`), Italian (`it`).

lang | done | missing
--- | --- | ---
de | 100% |  
eo | 86% | number_of_communities,preview,upload_image,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,theme,are_you_sure,yes,no 
es | 95% | archive_link,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default 
fr | 95% | archive_link,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default 
it | 96% | archive_link,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default 
nl | 88% | preview,upload_image,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,theme 
ru | 82% | cross_posts,cross_post,number_of_communities,preview,upload_image,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,recent_comments,theme,monero,by,to,transfer_community,transfer_site,are_you_sure,yes,no 
sv | 95% | archive_link,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default 
zh | 80% | cross_posts,cross_post,users,number_of_communities,preview,upload_image,formatting_help,view_source,sticky,unsticky,archive_link,settings,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,recent_comments,nsfw,show_nsfw,theme,monero,by,to,transfer_community,transfer_site,are_you_sure,yes,no 


If you'd like to update this report, run:

```bash 
cd ui
ts-node translation_report.ts > tmp # And replace the text above.
```

## Credits

Logo made by Andy Cuccaro (@andycuccaro) under the CC-BY-SA 4.0 license.
