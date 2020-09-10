# Configuration

The configuration is based on the file
[defaults.hjson](https://yerbamate.dev/LemmyNet/lemmy/src/branch/main/config/defaults.hjson).
This file also contains documentation for all the available options. To override the defaults, you
can copy the options you want to change into your local `config.hjson` file.

To use a different `config.hjson` location than the current directory, set the environment variable `LEMMY_CONFIG_LOCATION`. Make sure you copy the `defaults.hjson` if you do this, otherwise you will be missing settings.

Additionally, you can override any config files with environment variables. These have the same
name as the config options, and are prefixed with `LEMMY_`. For example, you can override the
`database.password` with `LEMMY_DATABASE__POOL_SIZE=10`.

An additional option `LEMMY_DATABASE_URL` is available, which can be used with a PostgreSQL
connection string like `postgres://lemmy:password@lemmy_db:5432/lemmy`, passing all connection
details at once.

If the Docker container is not used, manually create the database specified above by running the
following commands:

```bash
cd server
./db-init.sh
```
