# Contributing to BlogWriter

Thanks for your interest in contributing! 🎉

## Getting set up

1. Fork and clone the repository.
2. Follow the **Getting started** section in [README.md](README.md) to build
   the frontend and run the app.
3. Run the test suite before and after your changes:

   ```bash
   cargo test
   ```

## Reporting bugs

Open an issue using the bug report template. Please include:

- What you did, what you expected, and what happened
- OS and Rust version (`rustc --version`)
- Relevant log output (strip any API keys or tokens!)

## Suggesting features

Open an issue using the feature request template. Describe the use case, not
just the solution — it helps us find the best design.

## Pull requests

- Keep PRs focused: one feature or fix per PR.
- Add or update tests for behavior changes (cadence math, parsing, DB logic).
- Run `cargo fmt` and `cargo clippy` before submitting.
- If you change the command surface (`handlers.rs`), regenerate the typed
  bindings (`rata codegen`) and commit the updated `app/src/bindings.ts`.
- Update documentation (README, `docs/`) when behavior changes.

## Security

**Never commit secrets.** The local `blogwriter.db` contains API keys and site
tokens and is gitignored — keep it that way. If you discover a security issue,
please report it privately rather than opening a public issue (see
[SECURITY.md](SECURITY.md)).

## Code style

- Rust: idiomatic, `cargo fmt` formatted, no new clippy warnings.
- Svelte: follow the existing patterns in `app/src/App.svelte`.
- Commit messages: short imperative summary line (e.g. "Add weekly cadence
  jitter"), details in the body if needed.

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](LICENSE).
