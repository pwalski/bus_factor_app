# {{crate}} &emsp; [![build status](https://github.com/pwalski/bus_factor_app/actions/workflows/ci.yml/badge.svg)](https://github.com/pwalski/bus_factor_app/actions)[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE.md)

{{readme}}

## Examples

Sample run with default threshold (0.75).

```shell
cargo run -- --language rust --project-count 50
```

Use `--help` to check other params (like `--api-token`).

Add `RUST_LOG=debug` for verbose logs.

### Update of README.md

```shell
cargo install cargo-readme
cargo readme -r bus_factor -t ../README.tpl > README.md
```

---

License: {{license}}
