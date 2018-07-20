extern crate vmctrl;

use vmctrl::{local, ssh, CommandRunner};

fn main() {
    let l = local();

    let out = l.run_with_output("/bin/bash", &["-c", "ls"]).unwrap();

    for line in out {
        println!("local={:?}", line)
    }

    let r = ssh("macx");
    let out = r.run_with_output("/bin/bash", &["-c", "ls"]).unwrap();

    for line in out {
        println!("remote={:?}", line)
    }
}
