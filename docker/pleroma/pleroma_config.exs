# Pleroma instance configuration

import Config

config :pleroma, Pleroma.Web.Endpoint,
url: [host: "pleroma", scheme: "http", port: 4000],
http: [ip: {0, 0, 0, 0}, port: 4000],
secret_key_base: "0dqEgJ+GcXLVgcmMsya1nSf5DyiDy7lRkGqYKB/TyAxrrbzgcuxPKM+gloTrNJPL",
signing_salt: "GmRjWVZ9"

config :pleroma, :instance,
name: "pleroma:4000",
email: "chicken@example.com",
notify_email: "chicken@example.com",
limit: 5000,
registrations_open: true

config :pleroma, :media_proxy,
enabled: false,
redirect_on_failure: true
#base_url: "https://cache.pleroma.social"

config :pleroma, Pleroma.Repo,
adapter: Ecto.Adapters.Postgres,
username: "pleroma",
password: "hunter2",
database: "pleroma",
hostname: "postgres"

# Configure web push notifications
config :web_push_encryption, :vapid_details,
subject: "mailto:chicken@example.com",
public_key: "BDy9svG0DfHPzJwZBt4VBYS8ub_pId4-FUZQLXBcqmkYvZtYVnhbErJgViLYZROSIVVWY4U-sZgeMSNPJRVlt_g",
private_key: "BuPx7F7nd42VKejnW9U3yPPUPrlRbcgGCLfZcGETdgo"

config :pleroma, :database, rum_enabled: true
config :pleroma, :instance, static_dir: "/var/lib/pleroma/static"
config :pleroma, Pleroma.Uploaders.Local, uploads: "/var/lib/pleroma/uploads"

config :joken, default_signer: "UnyjyX3et+ImHWSVYJ3hCM5vexmB7wq6Zcx1qrv/GAGOZdBmq5/SKmX8jSWKB6xi"

config :pleroma, configurable_from_database: true

config :pleroma, Pleroma.Upload, filters: [Pleroma.Upload.Filter.Exiftool, Pleroma.Upload.Filter.AnonymizeFilename, Pleroma.Upload.Filter.Dedupe]

config :logger, :ex_syslogger,
  level: :debug