### Install build requirements
#### Ubuntu
```
sudo apt install git cargo libssl-dev pkg-config libpq-dev yarn curl gnupg2
# install yarn
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update && sudo apt install yarn
```

#### macOS

Install Rust using [the recommended option on rust-lang.org](https://www.rust-lang.org/tools/install) (rustup).

Then, install [Homebrew](https://brew.sh/) if you don't already have it installed.

Finally, install Node and Yarn.

```
brew install node yarn
```

### Get the source code
```
git clone https://github.com/LemmyNet/lemmy.git
# or alternatively from gitea
# git clone https://yerbamate.dev/LemmyNet/lemmy.git
```

All the following commands need to be run either in `lemmy/server` or `lemmy/ui`, as indicated
by the `cd` command.

### Build the backend (Rust)
```
cd server
cargo build
# for development, use `cargo check` instead)
```

### Build the frontend (Typescript)
```
cd ui
yarn
yarn build
```

### Setup postgresql
#### Ubuntu
```
sudo apt install postgresql
sudo systemctl start postgresql

# Either execute server/db-init.sh, or manually initialize the postgres database:
sudo -u postgres psql -c "create user lemmy with password 'password' superuser;" -U postgres
sudo -u postgres psql -c 'create database lemmy with owner lemmy;' -U postgres
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
```

#### macOS
```
brew install postgresql
brew services start postgresql
/usr/local/opt/postgres/bin/createuser -s postgres

# Either execute server/db-init.sh, or manually initialize the postgres database:
psql -c "create user lemmy with password 'password' superuser;" -U postgres
psql -c 'create database lemmy with owner lemmy;' -U postgres
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
```

### Run a local development instance
```
# run each of these in a seperate terminal
cd server && cargo run
cd ui && yarn start
```

Then open [localhost:4444](http://localhost:4444) in your browser. It will auto-refresh if you edit
any frontend files. For backend coding, you will have to rerun `cargo run`. You can use
`cargo check` as a faster way to find compilation errors.

To speed up incremental builds, you can add the following to `~/.cargo/config`:
```
[target.x86_64-unknown-linux-gnu]
rustflags = ["-Clink-arg=-fuse-ld=lld"]
```

Note that this setup doesn't include image uploads or link previews (provided by pict-rs and
iframely respectively). If you want to test those, you should use the
[Docker development](contributing_docker_development.md).
