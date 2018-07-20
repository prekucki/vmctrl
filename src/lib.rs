#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate nom;


pub trait Driver {
    type Error;
    type Machine: Machine<Error = Self::Error>;

    fn list_running(&self) -> Result<Vec<Self::Machine>, Self::Error>;

    fn from_path(&self, path: &str) -> Result<Self::Machine, Self::Error>;
}

pub trait Machine {
    type Error;

    fn name(&self) -> &str;

    fn list_snapshots(&self) -> Result<Vec<String>, Self::Error>;

    fn stop(&mut self) -> Result<(), Self::Error>;

    fn start(&mut self) -> Result<(), Self::Error>;

    fn revert_to<A : AsRef<str>>(&mut self, snapshot_name : A) -> Result<(), Self::Error>;

    fn create_snapshot<A :  AsRef<str>>(&mut self, snapshot_name: A) -> Result<(), Self::Error>;
}

pub use command::{local, ssh, CommandRunner};

pub mod command;
pub mod vmware;
pub mod virtual_box;