# Contributing to PSP Sidescreen

Thank you for wanting to help. This page is the short version; the
[engineering standards](docs/CONTRIBUTING.md) have the rules every change follows.

By taking part you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to help

- **Test on your hardware.** A PSP model or firmware that works (or does not) is the most useful
  thing you can tell us. Say which PSP, which firmware and what Windows version.
- **Report a bug** with the [bug form](https://github.com/rinmz/psp-sidescreen/issues/new/choose).
  The log of a failed run and the steps that lead to it save a lot of time.
- **Improve the documentation.** If a step confused you, it will confuse the next person.
- **Send code.** For anything bigger than a small fix, open an issue first so the direction can be
  agreed before you spend the time.

## Set up

You need Windows 10 or 11, [Rust](https://rustup.rs) (the version in `rust-toolchain.toml`) and
Node.js 24. [docs/BUILDING.md](docs/BUILDING.md) has the full steps, including the PSP side.

```
npm ci --prefix app
npm run build --prefix app
cargo test --workspace
```

## Before you open a pull request

Run the same checks as CI:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd app
npm run typecheck && npm run lint && npm run format:check && npm test && npm run build
```

Then:

- One logical change per pull request, with tests (a bug fix comes with a test that fails
  without it).
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org), for example
  `fix(host): keep the session alive when the monitor changes`. The subject is at most 72
  characters; the body says why.
- Say how you tried it. Anything that only the console can judge (capture, encoding, USB) should
  say which PSP and firmware it was tried on.

## Licence

By sending a contribution you agree that it is licensed under the [MIT licence](LICENSE) of this
project.
