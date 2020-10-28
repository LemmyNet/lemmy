# Configuration

The configuration is based on the file [defaults.hjson](https://yerbamate.dev/LemmyNet/lemmy/src/branch/main/config/defaults.hjson). This file also contains documentation for all the available options. To override the defaults, you can copy the options you want to change into your local `config.hjson` file.

The `defaults.hjson` and `config.hjson` files are located at `config/defaults.hjson` and`config/config.hjson`, respectively. To change these default locations, you can set these two environment variables:

    LEMMY_CONFIG_LOCATION           # config.hjson
    LEMMY_CONFIG_DEFAULTS_LOCATION  # defaults.hjson

Additionally, you can override any config files with environment variables. These have the same name as the config options, and are prefixed with `LEMMY_`. For example, you can override the `database.password` with `LEMMY_DATABASE__POOL_SIZE=10`.

An additional option `LEMMY_DATABASE_URL` is available, which can be used with a PostgreSQL connection string like `postgres://lemmy:password@lemmy_db:5432/lemmy`, passing all connection details at once.

If the Docker container is not used, manually create the database specified above by running the following commands:

```bash
cd server
./db-init.sh
```

### Snap Configuration

If you have installed Lemmy as a snap, configuration can be done via `snap set`. Each property in the `config.hjson` has a corresponding `snap set` parameter, with nested properties separated by dots. For example, to set the hostname of your Lemmy instance, run

    sudo snap set lemmy hostname=mylemmy

To set the admin username before setup, run

    sudo snap set lemmy setup.admin-username=admin

Note that each property name that contains an underscore in `config.hjson` must be set using a dash via `snap set`. The dash is replaced by an underscore in the generated `config.hjson`. For example, `sudo snap set lemmy setup.jwt-secret=foo` is equivalent to this `config.hjson`:

    {jwt_secret: "foo"}
