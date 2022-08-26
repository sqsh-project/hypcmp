# sqsh-benchmark

A way to benchmark the performance of the library. A configuration file
is being read for [`hyperfine`](https://github.com/sharkdp/hyperfine)
and then executed. A JSON file ist output.
Based on this, the performance can be analysed.

0. Define a `config.toml` document like in the [examples](./examples/) folder.
1. Read the configuration file.
2. For each `commit` execute
   a [subcommand](https://doc.rust-lang.org/std/process/struct.Command.html)
   with `hyperfine` with `hyperfine-parameters` and the `command-parameters`
   and save it as a JSON file.
3. Analyse and plot the JSON output using common plotting libraries.

> The output of each run will be written to a temporary directory.
> These will then be merged to a single json document.

## Configuration

```toml
label = "benchmark"
output = "benchmark.json"
hyperfine_params = [
   "--runs", "5",
   "--warmup", "3",
   "--parameter-list", "ifile", "Cargo.toml,README.md",
   "--parameter-list", "ofile", "/tmp/test.raw",
]

[run.past_versions]
commits = ["master", "asdfas"] # can be hash, tag or branch
command = "sqsh-cli duplicate --input {ifile} --output {ofile}"
hyperfine_params = [
   "--setup", "cargo install --path sqsh-cli",
   "--cleanup", "rm {ofile}"
]

[run.reference]
commits = ["sdfafs"]
command = "sqsh-cli duplicate --input {ifile} --output {ofile}"
hyperfine_params = [
   "--setup", "cargo install --path sqsh-cli",
   "--cleanup", "rm {ofile}"
]

[run.control]
command = "dd if={ifile} of={ofile}"
```
<!--
hyperfine --runs 50 -L commit e385914,master -L ifile Cargo.toml,Cargo.lock -L ofile /tmp/test.raw "dd if={ifile} of={ofile}" --warmup 3 --export-json /tmp/log.json --setup "git checkout {commit} && cargo install --path sqsh-cli" -n "{commit}-{ifile}-{ofile}" -->
