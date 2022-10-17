![MIT license](https://img.shields.io/crates/l/hypcmp)
![Version](https://img.shields.io/crates/v/hypcmp)
![Downloads](https://img.shields.io/crates/d/hypcmp)

# hypcmp

A way to benchmark applications with [`hyperfine`](https://github.com/sharkdp/hyperfine) using a TOML configuration 
file. This also enables benchmarking across commits using `commits = [...]` attribute.

> Benchmarking across commits is only possible if the git status is clean. 

## Examples

```toml
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

Further examples are given in the [examples](./examples/) folder.
