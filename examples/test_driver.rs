extern crate vmctrl;
use std::env;
use vmctrl::{Driver, Machine};

fn main() {
    let driver = vmctrl::driver();

    let args: Vec<String> = env::args().collect();

    println!("uri={}", args[1]);

    let m = driver.from_path(args[1].as_ref()).unwrap();

    println!("snapshots:");
    for s in m.list_snapshots().unwrap() {
        println!("s={}", s);
    }
}
