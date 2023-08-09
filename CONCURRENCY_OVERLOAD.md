# Rust Code Reference

https://www.sitepoint.com/rust-global-variables/

Four areas of entrance:

1. Admin backdoor should be on different pools for HTTP and PostgreSQL
2. RSS, disable when too busy?
3. API for clients
4. Federation API for peer servers

In a HTTP context of #3, #4 above, can we pass a variable around between session so that objects being built can consider the overload state and alter their logic?


# PostgreSQL concurrency

Self-awareness of overloaded PostgreSQL during peak activity periods....

Lemmy can support budget-orinted operatrs and some degrdation may be preferable to absolute errors.

1. Turn off account-specific blocking of communities, persons, etc
2. Turn off saved, read and other account-specific lookups
3. Turn off sort orders "Old", anything greater than 3 months, other unusual sorts
4. Logged-in account INSERT actions throttle back per-login
5. Creation of url post does outbound HTTP connection to fetch image/excerpt of posting. Disable under instance overload.
6. Turn off sign-up during instance overload?
7. Turn off community creation?

Right now there are hard-coded 24x7 throttles geared for performance. 300 posts, 50 on community list, etc.

Work In Progress, messy doc, needs more work. Got interrupted ;)
