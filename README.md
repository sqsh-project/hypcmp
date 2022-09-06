# hypcmp

A way to benchmark the performance of the library. A configuration file
is being read for [`hyperfine`](https://github.com/sharkdp/hyperfine)
and then executed. A JSON file ist output.
Based on this, the performance can be analysed.

## Configuration Example

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
commits = ["main"] # can be hash, tag or branch
command = "dd if={ifile} of={ofile}"
cleanup = "rm {ofile}"

[run.cp]
command = "cp {ifile} {ofile}"

[run.rsync]
command = "rsync -a {ifile} {ofile}"
```
