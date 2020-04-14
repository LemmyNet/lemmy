# Docker Development

## Running

```bash
git clone https://github.com/LemmyNet/lemmy
cd lemmy/docker/dev
./docker_update.sh # This builds and runs it, updating for your changes
```

and go to http://localhost:8536.

Note that compile times when changing `Cargo.toml` are relatively long with Docker, because builds can't be incrementally cached. If this is a problem for you, you should use [Local Development](contributing_local_development.md).
