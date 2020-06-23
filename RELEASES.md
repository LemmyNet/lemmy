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
[documentation](https://dev.lemmy.ml/docs/administration_backup_and_restore.html)
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

https://dev.lemmy.ml
