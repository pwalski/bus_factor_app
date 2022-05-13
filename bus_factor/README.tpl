[![build status](https://github.com/pwalski/bus_factor_app/actions/workflows/ci.yml/badge.svg)](https://github.com/pwalski/bus_factor_app/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE.md)

# {{crate}}

{{readme}}

## Examples

```shell
cargo build
```

Quick run with default threshold (0.75).

```shell
target/debug/bus_factor --language rust --project-count 10
```

Use `--help` to check other params (like `--api-token`).

### Update of README.md

```shell
cargo install cargo-readme
cargo readme --project-root bus_factor > README.md
```

---

License: {{license}}
