# Administration info

Information for Lemmy instance admins, and those who want to run a server.

## Install

Lemmy has two primary installation methods, [manually with Docker](install_docker.md), and [automated with Ansible](install_ansible.md). We recommend using Ansible, because it simplifies the installation and also makes updating easier.

### Manual install (without Docker)

Manual installs are *possible*, but not preferred, since Lemmy is dependent on other local services: The [lemmy-ui](https://github.com/LemmyNet/lemmy-ui), [a Postgresql Database](https://www.postgresql.org/), [pict-rs](https://git.asonix.dog/asonix/pict-rs/) for images, and [iframely](https://iframely.com/) for embeds. To see how these are wired together, look at the docker-compose.yml files. Due to the complexity of different systems, we will not support manual installs.
