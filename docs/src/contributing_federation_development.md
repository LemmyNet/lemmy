# Federation Development

## Setup

If you don't have a local clone of the Lemmy repo yet, just run the following command:

```bash
git clone https://yerbamate.dev/nutomic/lemmy.git -b federation
```

If you already have the Lemmy repo cloned, you need to add a new remote:
```bash
git remote add federation https://yerbamate.dev/nutomic/lemmy.git
git checkout federation
git pull federation federation
```

## Running

You need to have the following packages installed, the Docker service needs to be running.

- docker
- docker-compose
- cargo
- yarn

Then run the following
```bash
cd dev/federation-test
./run-federation-test.bash
```

After the build is finished and the docker-compose setup is running, open [127.0.0.1:8540](http://127.0.0.1:8540) and
[127.0.0.1:8541](http://127.0.0.1:8541) in your browser to use the test instances. You can login as admin with
username `lemmy` and password `lemmy`, or create new accounts.

Please get in touch if you want to contribute to this, so we can coordinate things and avoid duplicate work.