# Contributing to NetScene

Thank you for considering a contribution!

## Development Setup

1. Install Node.js dependencies with `npm install`.
2. Run the application using `npm run tauri dev`.

## Running Tests

Rust unit tests can be executed with:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

The JavaScript side currently has no automated tests.

## Coding Style

- Document public functions with Rustdoc comments.
- Prefer small, focussed commits.
- Ensure `cargo test` passes before opening a pull request.
