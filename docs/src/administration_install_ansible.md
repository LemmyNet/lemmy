First, you need to [install Ansible on your local computer](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html) (e.g. using `sudo apt install ansible`) or the equivalent for you platform.

Then run the following commands on your local computer:

```bash
git clone https://github.com/dessalines/lemmy.git
cd lemmy/ansible/
cp inventory.example inventory
nano inventory # enter your server, domain, contact email
ansible-playbook lemmy.yml --become
```
