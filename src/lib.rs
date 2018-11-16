#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;

pub trait Driver {
    type Machine: Machine;

    fn list_running(&self) -> Result<Vec<Self::Machine>, error::Error>;

    fn from_path(&self, path: &str) -> Result<Self::Machine, error::Error>;
}

pub trait Machine {
    fn name(&self) -> &str;

    fn list_snapshots(&self) -> Result<Vec<String>, error::Error>;

    fn stop(&mut self) -> Result<(), error::Error>;

    fn start(&mut self) -> Result<(), error::Error>;

    fn revert_to(&mut self, snapshot_name: &str) -> Result<(), error::Error>;

    fn create_snapshot(&mut self, snapshot_name: &str) -> Result<(), error::Error>;
}

pub use crate::command::{local, ssh, CommandRunner, FromCommandRunner};

pub mod command;
pub mod error;
pub mod uri;

#[cfg(feature = "virtualbox")]
pub mod virtual_box;
#[cfg(feature = "vmware")]
pub mod vmware;

mod remote;

pub fn driver() -> impl Driver<Machine = Box<Machine + 'static>> {
    let mut uri = uri::DriverRepo::default();

    #[cfg(feature = "vmware")]
    uri.register("vmware", vmware::local_driver());

    #[cfg(feature = "vmware")]
    uri.register("ssh+vmware", vmware::remote_driver());

    #[cfg(feature = "virtualbox")]
    uri.register("virtualbox", virtual_box::local_driver());

    #[cfg(feature = "virtualbox")]
    uri.register("ssh+virtualbox", virtual_box::remote_driver());

    uri
}
