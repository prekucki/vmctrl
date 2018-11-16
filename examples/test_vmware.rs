extern crate vmctrl;

use vmctrl::{local, vmware, Driver, FromCommandRunner, Machine};

fn main() {
    //let l = ssh("macx");
    let l = local();
    let local_vmware = vmware::factory().from_cmd(l);

    println!("running:");
    for v in local_vmware.list_running().unwrap() {
        println!("v={}", v.name())
    }

    /*println!("snapshots:");
    let mut m = local_vmware
        .from_path("/Users/prekucki/vm/ubuntu-a.vmwarevm/Ubuntu 64-bit.vmx")
        .unwrap();

    m.list_snapshots()
        .unwrap()
        .iter()
        .for_each(|m| println!("s={}", m));
    */
    //m.create_snapshot("test").unwrap();
    //m.revert_to("test").unwrap();

    let vb = vmware::factory().from_cmd(local());

    println!("vbox running:");
    for mut v in vb.list_running().unwrap() {
        println!("v={}", v.name());
        println!("snapshots:");
        let mut do_clean = false;
        for s in v.list_snapshots().unwrap() {
            println!("s={}", &s);
            if s == "clean" {
                do_clean = true;
            }
        }

        if do_clean {
            v.revert_to("clean").unwrap();
        }
    }
}
