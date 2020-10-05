# Docker Installation

Make sure you have both docker and docker-compose(>=`1.24.0`) installed. On Ubuntu, just run `apt install docker-compose docker.io`. Next, 

```bash
# create a folder for the lemmy files. the location doesnt matter, you can put this anywhere you want
mkdir /lemmy
cd /lemmy

# download default config files
wget https://raw.githubusercontent.com/LemmyNet/lemmy/main/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/LemmyNet/lemmy/main/docker/lemmy.hjson
wget https://raw.githubusercontent.com/LemmyNet/lemmy/main/docker/iframely.config.local.js

# Set correct permissions for pictrs folder
mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs
```

Open up your `docker-compose.yml`, and make sure `LEMMY_EXTERNAL_HOST` for `lemmy-ui` is set to your correct host.

```
- LEMMY_INTERNAL_HOST=lemmy:8536
- LEMMY_EXTERNAL_HOST=your-domain.com
- LEMMY_HTTPS=false
```

If you'd like a different database password, you should also change it in the `docker-compose.yml` **before** your first run.

After this, have a look at the [config file](administration_configuration.md) named `lemmy.hjson`, and adjust it, in particular the hostname, and possibly the db password. Then run:

`docker-compose up -d`

You can access the lemmy-ui at `http://localhost:1235`

To make Lemmy available outside the server, you need to setup a reverse proxy, like Nginx. [A sample nginx config](https://raw.githubusercontent.com/LemmyNet/lemmy/main/ansible/templates/nginx.conf), could be setup with:

```bash
wget https://raw.githubusercontent.com/LemmyNet/lemmy/main/ansible/templates/nginx.conf
# Replace the {{ vars }}
# The default lemmy_port is 8536
# The default lemmy_ui_port is 1235
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
```

You will also need to setup TLS, for example with [Let's Encrypt](https://letsencrypt.org/). After this you need to restart Nginx to reload the config.

## Updating

To update to the newest version, you can manually change the version in `docker-compose.yml`. Alternatively, fetch the latest version from our git repo:

```bash
wget https://raw.githubusercontent.com/LemmyNet/lemmy/main/docker/prod/docker-compose.yml
docker-compose up -d
```
