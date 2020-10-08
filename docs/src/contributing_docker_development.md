# Docker Development

## Dependencies (on Ubuntu)

```bash
sudo apt install git docker-compose
sudo systemctl start docker
git clone https://github.com/LemmyNet/lemmy
```

## Running

```bash
cd /docker/dev
./docker_update.sh
```

and go to http://localhost:1235.

*Note: many features (like docs and pictures) will not work without using an nginx profile like that in `ansible/templates/nginx.conf`.

To speed up the Docker compile, add the following to `/etc/docker/daemon.json` and restart Docker.
```
{
  "features": {
    "buildkit": true
  }
}
```

If the build is still too slow, you will have to use a
[local build](contributing_local_development.md) instead.
