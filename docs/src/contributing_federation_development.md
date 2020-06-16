# Federation Development

## Setup

If you don't have a local clone of the Lemmy repo yet, just run the following command:

```bash
git clone https://github.com/LemmyNet/lemmy -b federation
```

If you already have the Lemmy repo cloned, you need to add a new remote:
```bash
git remote add federation https://github.com/LemmyNet/lemmy
git checkout federation
git pull federation federation
```

## Running locally

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
[127.0.0.1:8550](http://127.0.0.1:8550) in your browser to use the test instances. You can login as admin with
username `lemmy_alpha` and `lemmy_beta` respectively, with password `lemmy`.

## Running on a server

Note that federation is currently in alpha. Only use it for testing, not on any production server, and be aware
that you might have to wipe the instance data at one point or another.

Follow the normal installation instructions, either with [Ansible](administration_install_ansible.md) or
[manually](administration_install_docker.md). Then replace the line `image: dessalines/lemmy:v0.x.x` in 
`/lemmy/docker-compose.yml` with `image: dessalines/lemmy:federation`. Also add the following in
`/lemmy/lemmy.hjson`:

```
    federation: {
        enabled: true
        allowed_instances: example.com
    }
```

Afterwards, and whenver you want to update to the latest version, run these commands on the server:

```
cd /lemmy/
sudo docker-compose pull
sudo docker-compose up -d
```
