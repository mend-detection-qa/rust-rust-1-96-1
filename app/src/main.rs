// probe-app: minimal binary stub.
use probe_core::{Config, render};

fn main() {
    env_logger::init();
    let cfg = Config { greeting: "hello".to_string() };
    match render("{ greeting } world", &cfg) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
