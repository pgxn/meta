PGXN Meta Spec
==============

The PGXN Meta Spec defines the requirements for the metadata file
(`META.json`) file for [PGXN] source distribution packages.

**[The specification can be found here](spec.md).**

## Contributing

Development of the spec takes place in the GitHub [pgxn/pgxn-meta-spec]
repository.

[Issues](/pgxn/pgxn-meta-spec/issues) are used for bugs and actionable items.
Longer discussions take place via [chat](#chat).

The specification and code is licensed under the PostgreSQL license found in
the [`LICENSE.md`](LICENSE.md) file in this repository.

Typos and grammatical errors can go straight to a pull-request. When in doubt,
start on the [mailing-list](#mailing-list).

### Testing

This project includes test written in [Rust]. Use this command to install
Rust:

``` sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once it's installed, run tests with `make test`.

### Linting

This project uses [pre-commit] to keep the code tidy and warning and
error-free. Install [pre-commit], use `make lint` to run the linters, and
`make .git/hooks/pre-commit` to force pre-commit to run before every commit.

## Chat

PGXN discussion happens in the following chat rooms:

*   [PGXN Discussions] (best for design discussions)
*   `#extensions` channel on [Postgres Slack]
*   `#extension-discuss` channel on [Postgres Discord]

  [PGXN]: https://pgxn.org "PGXN: PostgreSQL Extension Network"
  [pgxn/pgxn-meta-spec]: https://github.com/pgxn/pgxn-meta-spec
  [Rust]: https://www.rust-lang.org "The Rust Programming Language"
  [pre-commit]: https://pre-commit.com "A framework for managing and maintaining multi-language pre-commit hooks."
  [Postgres Slack]: https://pgtreats.info/slack-invite
  [Postgres Discord]: https://discord.com/invite/bW2hsax8We
  [PGXN Discussions]: https://github.com/orgs/pgxn/discussions/
