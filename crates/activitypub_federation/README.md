Activitypub-Federation
===

A high-level framework for [ActivityPub](https://www.w3.org/TR/activitypub/) federation in Rust, extracted from [Lemmy](https://join-lemmy.org/). The goal is that this library can take care of almost everything related to federation for different projects, but for now it is still far away from that goal.

## Features

- ObjectId type, wraps the `id` url and allows for type safe fetching of objects, both from database and HTTP
- Queue for activity sending, handles HTTP signatures, retry with exponential backoff, all in background workers
- Inbox for receiving activities, verifies HTTP signatures, performs other basic checks and helps with routing
- Generic error type (unfortunately this was necessary)
- various helpers for verification, (de)serialization, context etc

## Roadmap

Things to work on in the future:
- **Simplify generics**: The library uses a lot of generic parameters, where clauses and associated types. It should be possible to simplify them.
- **Improve macro**: The macro is implemented very badly and doesn't have any error handling.
- **Generate HTTP endpoints**: It would be possible to generate HTTP endpoints automatically for each actor.
- **Support for other web frameworks**: Can be implemented using feature flags if other projects require it.
- **Signed fetch**: JSON can only be fetched by authenticated actors, which means that fetches from blocked instances can also be blocked. In combination with the previous point, this could be handled entirely in the library.
- **Helpers for testing**: Lemmy has a pretty useful test suite which (de)serializes json from other projects, to ensure that federation remains compatible. Helpers for this could be added to the library.
- **[Webfinger](https://datatracker.ietf.org/doc/html/rfc7033) support**: Not part of the Activitypub standard, but often used together for user discovery.
- **Remove request_counter from API**: It should be handled internally and not exposed. Maybe as part of `Data` struct.

## How to use

To get started, have a look at the example. If anything is unclear, please open an issue for clarification. You can also look at [Lemmy code](https://github.com/LemmyNet/lemmy/tree/main/crates/apub) for a more advanced implementation.

## License

[AGPLv3](../../LICENSE)