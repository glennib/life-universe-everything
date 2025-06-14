# Life, the Universe and Everything

Simple demographics simulator.

See https://glennib.github.io/life-universe-everything/.

![screenshot of GUI](./.assets/screenshot.png)

To build you need Rust/Cargo: https://rustup.rs/.

```shell
cargo run --release
```

GUI is built with egui/[eframe](https://github.com/emilk/egui/tree/main/crates/eframe).
Check out that link if you need help with GUI-specific system dependencies.
This is probably only needed if you use Linux.

Deploy to Github Pages:

```shell
trunk build --release --dist docs --public-url /life-universe-everything/
```

Commit the `docs/` directory to deploy.
