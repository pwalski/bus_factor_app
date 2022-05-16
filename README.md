# bus_factor &emsp; [![build status](https://github.com/pwalski/bus_factor_app/actions/workflows/ci.yml/badge.svg)](https://github.com/pwalski/bus_factor_app/actions)[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE.md)

Bus factor estimation

## Overview

Bus factor is a measurement which attempts to estimate the number of key persons a repository would need to lose in order for it to become stalled due to lack of expertise.
It is commonly used in the context of software development.
For example, if a given repository is developed by a single person, then the repository's bus factor is equal to 1 (it's likely for the repository to become unmaintained if the main contributor suddenly stops working on it).

Library finds popular GitHub repositories with a bus factor of 1.
Given a programming language name (`language`) and a repository count (`repo_count`), library fetches the first `repo_count` most popular repositories (sorted by the number of GitHub stars) from the given language.
Then, for each repository, it inspect its contributor statistics.
We assume a repository's bus factor is 1 if its most active developer's contributions account for 75% or more of the total contributions count from the top 25 most active developers.
Repositories with a bus factor of 75% or higher are returned as a Result.

## Examples

Sample run with default threshold (0.75).

```shell
RUST_LOG=info cargo run -- --language rust --project-count 50
```

Use `--help` to check other params (like `--api-token`).

### Update of README.md

```shell
cargo install cargo-readme
cargo readme -r bus_factor -t ../README.tpl > README.md
```

---

License: MIT
