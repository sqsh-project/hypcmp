![MIT license](https://img.shields.io/crates/l/hypcmp)
![Version](https://img.shields.io/crates/v/hypcmp)
![Downloads](https://img.shields.io/crates/d/hypcmp)

# hypcmp

A way to benchmark the performance of different applications with [`hyperfine`](https://github.com/sharkdp/hyperfine) using a TOML configuration file and JSON output. Further, each run can include a `[commits] = [...]` tag, which enables
comparing across commits.

> Be aware that the git status of the repository is not allowed to be dirty for commits to work, since a checkout is being executed in the background.

## Usage

Simple feed a configuration file for the benchmark:

```bash
hypcmp examples/duplicate.toml
```

## Example configurations

```toml
output = "copy.json"
hyperfine_params = [  # common hyperfine parameters for all runs
    "--runs", "5",
    "--warmup", "3",
    "--style", "none",
]

[run.dd]
command = "dd if=Cargo.toml of=/tmp/Cargo.toml.dd"

[run.cp]
command = "cp Cargo.toml /tmp/Cargo.toml.cp"

[run.rsync]
command = "rsync -a Cargo.toml /tmp/Cargo.toml.rsync"
```

A more complicated example:

```toml
output = "duplicate.json"
hyperfine_params = [  # common hyperfine parameters for all runs
    "--runs", "5",
    "--warmup", "3",
    "--parameter-list", "ifile", "Cargo.toml,README.md",
    "--parameter-list", "ofile", "/tmp/test.raw",
]

[run.dd]
commits = ["main"] # can be full commit id, an abbreviated id w/ 7 letters, tag or branch
command = "dd if={ifile} of={ofile}"
cleanup = "rm {ofile}"

[run.cp]
command = "cp {ifile} {ofile}"

[run.rsync]
command = "rsync -a {ifile} {ofile}"
```

Other examples are given in the [examples](./examples/) folder.
