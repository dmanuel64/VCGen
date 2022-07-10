# Vulnerable Code Generator
The Vulnerable Code Generator (`VCGen`) is an efficient, clean, and fast tool used to generate datasets containing vulnerable code entries from thousands of real-world public repositories from GitHub containing C code.

`VCGen` is able to identify likely vulnerable commits and use various source code static analyzers to scan C files and determine if it contains any vulnerabilities.

Once all vulnerable entries are identified, an organized dataset is generated into a JSON lines (`.jsonl`) file.

## Installation
`VCGen` can be built using `cargo`. Some optional scripts require `python3` and extra `pip` libraries.

All additional pre-requisites can be installed automatically with the `scripts/install_prereqs.bash` script. At a minimum, `VCGen` requires one acceptable source code static analyzer and `libssl-dev`.

Once the repository is cloned, `VCGen` can be installed with:
```
cargo install --path <path_to_local_repo>
```
and executed with the `vcgen` executable.

### Supported Static Analyzers
- [Flawfinder](https://dwheeler.com/flawfinder/)