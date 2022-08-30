# sqsh-benchmark

A way to benchmark the performance of the library. A configuration file
is being read for [`hyperfine`](https://github.com/sharkdp/hyperfine)
and then executed. A JSON file ist output.
Based on this, the performance can be analysed.

## Configuration

An example:

```toml
label = "duplicate"
output = "duplicate.json"
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
label = "duplicate"
output = "duplicate.json"
hyperfine_params = [  # common hyperfine parameters for all runs
    "--runs", "5",
    "--warmup", "3",
    "--parameter-list", "ifile", "Cargo.toml,README.md",
    "--parameter-list", "ofile", "/tmp/test.raw",
]

[run.past]
commits = ["master", "asdfas"]  # can be hash, tag or branch
command = "sqsh-cli duplicate --input {ifile} --output {ofile}"
setup = "cargo install --release --path ."
cleanup = "rm {ofile}"
prepare = "sync; echo 3 | sudo tee /proc/sys/vm/drop_caches"

[run.reference]
commits = ["sdfafs"]
command = "sqsh-cli duplicate --input {ifile} --output {ofile}"
setup = "cargo install --release --path ."

[run.control]
command = "dd if={ifile} of={ofile}"
```
