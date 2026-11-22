# Contributing

Each app is a separate Rust workspace. Clone this repository and install a current stable Rust toolchain and your platform’s native compiler/linker. Start with the component you want to change. Read its AGENTS.md, Cargo.toml, notices, and platform build instructions. Some builds fetch pinned dependencies or use optional fonts and translation inputs.

For example, from the repository root:

```sh
cd tz-cellforge
cargo test --locked -p gridcraft-engine
cargo build --locked --release -p gridcraft --bin tz-cellforge
```

For UI bugs, include the app name, version, window dimensions, display scaling, steps, and a screenshot with personal data removed. For file bugs, attach a small original test file and describe the expected round trip. For packaging work, include the OS and architecture, package format, and install/launch/uninstall results.

Submit focused changes from a branch and explain how you tested them. Friendly first contributions include clarifying a guide, reporting a reproducible layout problem, or adding a regression test. Preserve existing copyright notices and identify the license of any added assets.
