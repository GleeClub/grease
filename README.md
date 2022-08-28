Grease
------

The [GraphQL][graphql] API for the Georgia Tech Glee Club's official site, [GlubHub][glubhub].
This API is a wrapper over a [PostgreSQL][psql] database and manages everything the Glee Club
needs to run successfully. This includes managing events, attendance, carpool, officer notes,
music, and more. It also runs an email loop to send helpful emails about new and upcoming events
to all members.

## Usage

The API is hosted at <https://api.glubhub.org>. Just send a POST request to the API and optionally
set your login token as header `GREASE_TOKEN` to authenticate as a user after logging in.

Since Grease runs on GraphQL, visiting the API in your browser will give you a GraphiQL instance
with interactive documentation and a query maker. If you pass `?token=<your token>` at the end of
the URL, it will automatically set your `GREASE_TOKEN` with every request. All queries, mutations,
and object types are fully documented there.

## How It's Hosted

In "production", the app runs on [fly.io][flyio]. fly.io takes docker images and runs them
continuously like a server. You can manually

## Development

If you wanna work on this project



















## Development

To work on this project, you'll need to have [Rust Nightly][install rust] installed. Once you install
rust on the `nightly` toolchain command:

```bash
rustup target add x86_64-unknown-linux-musl
```

Once you have that installed, you can use the [simple build script](./build_and_upload.sh)
to build the API with full optimization and upload it to the web hosting platform.
(You'll need to be on Georgia Tech's network or the official GT VPN to upload over scp
for now.)


[fly.io]: https://fly.io/
[api]: https://api.glubhub.org/
[graphql]: https://graphql.org/
[glubhub]: https://glubhub.org/
[rust]: https://www.rust-lang.org/
[psql]: https://www.postgresql.org/
[install rust]: https://www.rust-lang.org/learn/get-started
