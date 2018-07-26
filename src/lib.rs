#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;

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

    fn revert_to(&mut self, snapshot_name: &str) -> Result<(), Self::Error>;

    fn create_snapshot(&mut self, snapshot_name: &str) -> Result<(), Self::Error>;
}

pub use command::{local, ssh, CommandRunner};

pub mod command;
pub mod uri;
pub mod error;

#[cfg(feature="virtualbox")]
pub mod virtual_box;
#[cfg(feature="vmware")]
pub mod vmware;

pub fn driver() -> impl Driver<Error=error::Error,Machine=Box<Machine<Error=error::Error> + 'static>> {
    let mut uri = uri::DriverRepo::default();

    #[cfg(feature="vmware")]
    uri.register("vmware", vmware::local_driver());

    #[cfg(feature="virtualbox")]
    uri.register("virtualbox", virtual_box::local_driver());

    uri
}



