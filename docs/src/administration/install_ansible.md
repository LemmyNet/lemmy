# Ansible Installation

This is the same as the [Docker installation](install_docker.md), except that Ansible handles all of it automatically. It also does some extra things like setting up TLS and email for your Lemmy instance.

First, you need to [install Ansible on your local computer](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html) (e.g. using `sudo apt install ansible`) or the equivalent for you platform.

Then run the following commands on your local computer:

```bash
git clone https://github.com/LemmyNet/lemmy.git
cd lemmy/ansible/
cp inventory.example inventory
nano inventory # enter your server, domain, contact email
# If the command below fails, you may need to comment out this line
# In the ansible.cfg file:
# interpreter_python=/usr/bin/python3
ansible-playbook lemmy.yml --become
```

To update to a new version, just run the following in your local Lemmy repo:
```bash
git pull origin main
cd ansible
ansible-playbook lemmy.yml --become
```
