# Docker Installation

Make sure you have both docker and docker-compose(>=`1.24.0`) installed. On Ubuntu, just run `apt install docker-compose docker.io`. Next, 

```bash
# create a folder for the lemmy files. the location doesnt matter, you can put this anywhere you want
mkdir /lemmy
cd /lemmy

# download default config files
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/lemmy.hjson
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/iframely.config.local.js

# Set correct permissions for pictrs folder
mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs
```

After this, have a look at the [config file](administration_configuration.md) named `lemmy.hjson`, and adjust it, in particular the hostname, and possibly the db password. Then run:

`docker-compose up -d`

To make Lemmy available outside the server, you need to setup a reverse proxy, like Nginx. [A sample nginx config](https://raw.githubusercontent.com/dessalines/lemmy/master/ansible/templates/nginx.conf), could be setup with:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/ansible/templates/nginx.conf
# Replace the {{ vars }}
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
```

You will also need to setup TLS, for example with [Let's Encrypt](https://letsencrypt.org/). After this you need to restart Nginx to reload the config.

## Updating

To update to the newest version, you can manually change the version in `docker-compose.yml`. Alternatively, fetch the latest version from our git repo:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
docker-compose up -d
```
