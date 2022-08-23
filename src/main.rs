use std::{fs::File, io::Read};
use toml::Value;

fn main() {
    let filename = "./sqsh-benchmark/examples/base.toml";
    let mut f = File::open(filename).unwrap();
    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();
    let value = content.as_str().parse::<Value>().unwrap();
    println!("{value:?}");
}
