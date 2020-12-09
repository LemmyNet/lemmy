# Federation

Lemmy uses the ActivityPub protocol (a W3C standard) to enable federation between different servers (often called instances). This is very similar to the way email works. For example, if you use gmail.com, then you can not only send mails to other gmail.com users, but also to yahoo.com, yandex.ru and so on. Email uses the SMTP protocol to achieve this, so you can think of ActivityPub as "SMTP for social media". The amount of different actions possible on social media (post, comment, like, share, etc) means that ActivityPub is much more complicated than SMTP.

As with email, ActivityPub federation happens only between servers. So if you are registered on `enterprise.lemmy.ml`, you only connect to the API of `enterprise.lemmy.ml`, while the server takes care of sending and receiving data from other instances (eg `voyager.lemmy.ml`). The great advantage of this approach is that the average user doesn't have to do anything to use federation. In fact if you are using Lemmy, you are likely already using it. One way to confirm is by going to a community or user profile. If you are on `enterprise.lemmy.ml` and you see a user like `@nutomic@voyager.lemmy.ml`, or a community like `!main@ds9.lemmy.ml`, then those are federated, meaning they use a different instance from yours.

One way you can take advantage of federation is by opening a different instance, like `ds9.lemmy.ml`, and browsing it. If you see an interesting community, post or user that you want to interact with, just copy its URL and paste it into the search of your own instance. Your instance will connect to the other one (assuming the allowlist/blocklist allows it), and directly display the remote content to you, so that you can follow a community or comment on a post. Here are some examples of working searches:

- `!main@lemmy.ml` (Community)
- `@nutomic@lemmy.ml` (User)
- `https://lemmy.ml/c/programming` (Community)
- `https://lemmy.ml/u/nutomic` (User)
- `https://lemmy.ml/post/123` (Post)
- `https://lemmy.ml/comment/321` (Comment)

You can see the list of linked instances by following the "Instances" link at the bottom of any Lemmy page.