### Ubuntu


#### Build requirements:
```
sudo apt install git cargo libssl-dev pkg-config libpq-dev yarn curl gnupg2 git
# install yarn
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update && sudo apt install yarn
```

#### Get the source code
```
git clone https://github.com/LemmyNet/lemmy.git
# or alternatively from gitea
# git clone https://yerbamate.dev/LemmyNet/lemmy.git
```

All the following commands need to be run either in `lemmy/server` or `lemmy/ui`, as indicated
by the `cd` command.

#### Build the backend (Rust)
```
cd server
cargo build
# for development, use `cargo check` instead)
```

#### Build the frontend (Typescript)
```
cd ui
yarn
yarn build
```

#### Setup postgresql
```
sudo apt install postgresql
sudo systemctl start postgresql
# initialize postgres database
sudo -u postgres psql -c "create user lemmy with password 'password' superuser;" -U postgres
sudo -u postgres psql -c 'create database lemmy with owner lemmy;' -U postgres
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
# or execute server/db-init.sh
```

#### Run a local development instance
```
# run each of these in a seperate terminal
cd server && cargo run
ui & yarn start
```

Then open [localhost:4444](http://localhost:4444) in your browser. It will auto-refresh if you edit
any frontend files. For backend coding, you will have to rerun `cargo run`. You can use
`cargo check` as a faster way to find compilation errors.

Note that this setup doesn't include image uploads or link previews (provided by pict-rs and
iframely respectively). If you want to test those, you should use the
[Docker development](contributing_docker_development.md).
