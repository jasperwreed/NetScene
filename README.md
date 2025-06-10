# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Network Scanning

This app exposes a command `scan_network` that lists devices found in the local
ARP table. It can be triggered from the React UI using the **Scan Network**
button.

## Development

To install dependencies and run the application:

```bash
npm install
npm run tauri dev
```

### Running Tests

The Rust backend contains unit tests for the ARP parsing logic. Run them with:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines when submitting patches.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
