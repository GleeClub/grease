Grease
------

The [GraphQL][graphql] API for the Georgia Tech Glee Club's internal site, [GlubHub][glubhub].
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

## Development

This API is written in [Rust][rust], which you can [install here][install rust]. You'll also need to
install [flyctl][flyctl] so that you can proxy the fly.io-hosted PostgreSQL database to your local
machine so [sqlx][sqlx] can typecheck SQL queries against it. To proxy the primary PostgreSQL database
to your local machine, login to fly.io:

```bash
fly auth login
```

And then run in a separate terminal:

```bash
fly proxy 5432:5432 -a grease-db
```

If you're not yet part of the `gleeclub` organization on fly.io, email Sam Mohr at sam.mohr@protonmail.com
to get added to the organization so that you can access the [fly.io dashboard][fly.io dashboard].

## Hosting and Deployment

In "production", the app runs on [fly.io][fly.io]. fly.io takes docker images and runs them
continuously like a server. You can manually deploy code there with `fly deploy`, but it's easier
to let this repo's [GitHub Action][deploy action] do it.

Automatic deployment happens on push to remote by using the official GitHub Actions step
from fly.io with credentials saved to the GitHub Secrets for this repo. All you need to do
is push your updated code to GitHub, and if it works, it'll update the fly.io instance, and
rollback to the previous version if not.

To make sure the code works, you'll want to make sure it's formatted with `cargo fmt` and that the
code (including SQL queries) is correct with `cargo sqlx prepare`. You can install the `cargo sqlx`
subcommand with `cargo install sqlx-cli`. To automatically make sure that you're good to go before
you commit anything, you can run the following command to make sure everything is formatted and
typechecking correctly:

```bash
printf "#!/bin/sh\n\ncargo fmt && cargo sqlx prepare" > .git/hooks/pre-commit
```


[fly.io]: https://fly.io/
[fly.io dashboard]: https://fly.io/apps/grease
[graphql]: https://graphql.org/
[glubhub]: https://github.com/GleeClub/glubhub
[psql]: https://www.postgresql.org/
[rust]: https://www.rust-lang.org/
[install rust]: https://www.rust-lang.org/learn/get-started
[flyctl]: https://fly.io/docs/flyctl/installing/
[sqlx]: https://github.com/launchbadge/sqlx
[deploy action]: ./.github/workflows/deploy.yml
