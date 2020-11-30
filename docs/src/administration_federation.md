# Federation 

Note: ActivityPub federation is still under development. We recommend that you only enable it on test instances for now.

To enable federation, change the setting `federation.enabled` to `true` in `lemmy.hjson`, and restart Lemmy.

Federation does not start automatically, but needs to be triggered manually through the search. To do this you have to enter a reference to a remote object, such as:

- `!main@lemmy.ml` (Community)
- `@nutomic@lemmy.ml` (User)
- `https://lemmy.ml/c/programming` (Community)
- `https://lemmy.ml/u/nutomic` (User)
- `https://lemmy.ml/post/123` (Post)

For an overview of how federation in Lemmy works on a technical level, check out our [Federation Overview](contributing_federation_overview.md).

## Instance allowlist and blocklist

The federation section of Lemmy's config has two variables `allowed_instances` and `blocked_instances`. These control which other instances Lemmy will federate with. Both settings take a comma separated list of domains, eg `lemmy.ml,example.com`. You can either change those settings via `/admin`, or directly on the server filesystem. 

It is important to note that these settings only affect sending and receiving of data between instances. If allow federation with a certain instance, and then remove it from the allowlist, this will not affect previously federated data. These communities, users, posts and comments will still be shown. They will just not be updated anymore. And even if an instance is blocked, it can still fetch and display public data from your instance.

By default, both `allowed_instances` and `blocked_instances` values are empty, which means that Lemmy will federate with every compatible instance. We do not recommend this, because the moderation tools are not yet ready to deal with malicious instances.

What we do recommend is putting a list of trusted instances into `allowed_instances`, and only federating with those. Note that both sides need to add each other to their `allowed_instances` to allow two-way federation.

Alternatively you can also use blocklist based federation. In this case, add the domains of instances you do *not* want to federate with. You can only set one of `allowed_instances` and `blocked_instances`, as setting both doesn't make sense.
