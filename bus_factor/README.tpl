[![build status](https://github.com/pwalski/bus_factor_app/actions/workflows/ci.yml/badge.svg)](https://github.com/pwalski/bus_factor_app/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE.md)

# {{crate}}

{{readme}}

## Examples

```shell
cargo build
```

Quick run with low threshold (default 0.75 requires many projects to check)

```shell
target/debug/bus_factor --language rust --project-count 5 --threshold 0.3
```

### Update of README.md

```shell
cargo install cargo-readme
cargo readme --project-root bus_factor > README.md
```

---

License: {{license}}
