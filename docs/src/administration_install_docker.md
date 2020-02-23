# Docker Installation

Make sure you have both docker and docker-compose(>=`1.24.0`) installed:

```bash
mkdir lemmy/
cd lemmy/
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/lemmy.hjson
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/iframely.config.local.js
# Edit lemmy.hjson, and docker-compose.yml to do more configuration (like adding a custom password)
docker-compose up -d
```

and go to http://localhost:8536.

[A sample nginx config](/ansible/templates/nginx.conf) (Note: Avatar / Image uploading won't work without this), could be setup with:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/ansible/templates/nginx.conf
# Replace the {{ vars }}
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
```
## Updating

To update to the newest version, run:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
docker-compose up -d
```
