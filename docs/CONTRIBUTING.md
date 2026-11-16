# Engineering Standards

These rules apply to every change, regardless of who writes it.

## Rust

- **Small, focused modules.** One responsibility per file. Public API surfaces stay minimal.
- **No dead code, no speculation.** Add code when a milestone needs it, not before.
- **Errors are values.** Library crates return typed errors and never panic on input from the
  USB link, the network or the OS. `unwrap` and `expect` are allowed only in tests and where the
  invariant is stated in a comment.
- **No `unsafe` unless unavoidable.** `unsafe_code` is denied workspace-wide. Only modules that
  call Windows APIs (capture, USB) opt in with `#![allow(unsafe_code)]`, stay small, and document
  the safety contract of every `unsafe` block.
- **Comments explain why**, not what. Public items carry doc comments, including `# Errors`.
- **Dependencies are a cost.** Add one only with a clear reason and mention it in the commit.

## C (PSP side)

- Built with PSPSDK, `-Wall -Wextra -Werror`, C11.
- No dynamic allocation in the per-frame path. Buffers are allocated once at start-up.
- Every call into the PSP OS checks its return value; errors are reported on screen, not
  ignored.
- Naming: `snake_case` functions prefixed with their module (`usb_`, `video_`, `net_`),
  `UPPER_SNAKE` constants, one header per module.

## Window (TypeScript, `app/`)

- Strict TypeScript, no `any`. Components are small and do one job; the state of the window lives
  in one hook (`useSidescreen`) and the program behind it is reached only through the `Backend`
  interface, which has a stand-in so the window can be checked in a plain browser.
- Colours come only from `src/styles/theme.css`. A hex value appears there once, as a
  light/dark pair, and components use the tokens (`bg-brand`, `text-ink-2`, ...).
- Every control has an accessible name and a visible focus ring; motion respects
  `prefers-reduced-motion`.
- The look follows [Handy](https://github.com/cjpais/Handy) (MIT): a sidebar, titled groups of
  settings rows, a status bar. The yellow is the logo's.
- Gates, run in `app/`: `npm run typecheck`, `npm run lint`, `npm run format:check`,
  `npm run build`.

## Tests

- Protocol and framing code is covered by unit tests, including malformed and truncated input.
- A bug fix includes a test that fails without the fix.
- Hardware-dependent behaviour (capture, encode, USB) sits behind small interfaces so the logic
  around it is testable without hardware.
- Behaviour that can only be judged on the console is recorded as a measurement in
  `docs/MEASUREMENTS.md`, with the date, firmware and conditions.

## Tooling gates

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Commits

- One logical change per commit; every commit builds and passes tests.
- [Conventional Commits](https://www.conventionalcommits.org): `type(scope): summary`, imperative
  mood, at most 72 characters, no trailing period. Types: `feat`, `fix`, `refactor`, `perf`,
  `test`, `docs`, `build`, `ci`, `chore`.
- The body explains why, and any trade-offs, wrapped at 72 columns.
- No generated or tool attribution lines.

## Protocol changes

A change to the wire protocol updates `docs/PROTOCOL.md`, the `sidescreen-proto` crate with
tests, the C implementation and the version number in one commit.
