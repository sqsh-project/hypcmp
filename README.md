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
