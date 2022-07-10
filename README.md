# Vulnerable Code Generator
The Vulnerable Code Generator (`VCGen`) is an efficient, clean, and fast tool used to generate datasets containing vulnerable code entries from thousands of real-world public repositories from GitHub containing C code.

`VCGen` is able to identify likely vulnerable commits and use various source code static analyzers to scan C files and determine if it contains any vulnerabilities.

Once all vulnerable entries are identified, an organized dataset is generated into a JSON lines (`.jsonl`) file.

## Installation
`VCGen` can be built using `cargo`. Some optional scripts require `python3` and extra `pip` libraries.

All additional pre-requisites can be installed automatically with the `scripts/install_prereqs.bash` script. At a minimum, `VCGen` requires one supported source code static analyzer and `libssl-dev`.

Once the repository is cloned, `VCGen` can be installed with:
```
cargo install --path <path_to_local_repo>
```
and executed with the `vcgen` executable.

### Supported Static Analyzers
The following are supported source code static analyzers that can be used by `VCGen`:
- [Flawfinder](https://dwheeler.com/flawfinder/)

## Scripts
The following are utility scripts in the `scripts` directory:
- `install_prereqs.bash`: installs `libssl-dev` and Flawfinder. Must be ran as root
- `vcgen_excel.py`: wrapper script for `vcgen` that additionally creates an Excel Spreadsheet (`.xlsx`) from the JSON lines output in the same directory
- `inspector.py`: utility script to be used with a JSON lines vulnerability dataset that steps through each entry, printing out the associated vulnerability message and surrounding code. Useful for manual review of data entry validity