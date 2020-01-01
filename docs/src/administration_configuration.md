The configuration is based on the file [defaults.hjson](server/config/defaults.hjson). This file also contains documentation for all the available options. To override the defaults, you can copy the options you want to change into your local `config.hjson` file. 

Additionally, you can override any config files with environment variables. These have the same name as the config options, and are prefixed with `LEMMY_`. For example, you can override the `database.password` with 
`LEMMY__DATABASE__POOL_SIZE=10`.

An additional option `LEMMY_DATABASE_URL` is available, which can be used with a PostgreSQL connection string like `postgres://lemmy:password@lemmy_db:5432/lemmy`, passing all connection details at once.
