# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Network Scanning

This app exposes a command `scan_network` that lists devices found in the local
ARP table. Internally it shells out to the system command `arp -a`. The command
is triggered from the React UI using the **Scan Network** button.

### Platform Notes

Running `arp -a` may require different privileges depending on the operating
system. On Linux and macOS, elevated permissions might be necessary to read the
ARP table. Windows also relies on `arp -a`, which may require Administrator
rights in certain environments. If you run into permission errors, try launching
the application with the appropriate privileges for your OS.

## Pi-hole Statistics

Another command `get_pihole_stats` fetches statistics from a Pi-hole instance
using its FTL API. The React UI provides a **Get Pi-hole Stats** button that
shows information such as domains blocked and queries today.

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

## License

This project is licensed under the [MIT License](LICENSE).

