extern crate vmctrl;

use vmctrl::{local, ssh, vmware, CommandRunner, Driver, Machine, virtual_box};

fn main() {
    //let l = ssh("macx");
    let l = local();
    let local_vmware = vmware::Driver::from_cmd(l);

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


    let vb = virtual_box::Driver::from_cmd(local());


    println!("vbox running:");
    for v in vb.list_running().unwrap() {
        println!("v={}", v.name())
    }


}
