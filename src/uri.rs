use super::Driver;
use super::Machine;
use std::any::Any;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use super::error::*;

struct VmUri<'a> {
    schema: &'a str,
    path: &'a str,
}

impl<'a> From<&'a str> for VmUri<'a> {
    fn from(uri: &'a str) -> Self {
        if let Some(pos) = uri.find(':') {
            VmUri {
                schema: &uri[0..pos],
                path: &uri[pos + 1..],
            }
        } else {
            VmUri {
                schema: "file",
                path: uri,
            }
        }
    }
}

type MachinePtr = Box<Machine<Error = Error>>;

pub trait DriverFactory {
    fn machine_for_uri(&self, uri: &str) -> Option<MachinePtr>;
}

#[derive(Clone, Default)]
pub struct DriverRepo {
    inner: Rc<RefCell<DriverRepoImpl>>,
}

#[derive(Default)]
struct DriverRepoImpl {
    scheme: HashMap<&'static str, Box<DriverFactory>>,
}

impl DriverRepo {
    pub fn register(&mut self, scheme: &'static str, factory: Box<DriverFactory>) {
        let s = &mut self.inner.borrow_mut().scheme;

        s.insert(scheme, factory);
    }

    pub fn apply<Fn, T>(&self, scheme: &str, f: Fn) -> Option<T>
    where
        Fn: FnOnce(&DriverFactory) -> Option<T>,
    {
        if let Some(driver_factory) = self.inner.borrow().scheme.get(scheme) {
            f(driver_factory.as_ref())
        } else {
            None
        }
    }
}

impl Driver for DriverRepo {
    type Error = Error;
    type Machine = MachinePtr;

    fn list_running(&self) -> Result<Vec<<Self as Driver>::Machine>> {
        unimplemented!()
    }

    fn from_path(&self, path: &str) -> Result<<Self as Driver>::Machine> {
        let uri: VmUri = path.into();

        if let Some(machine) = self.apply(uri.schema, |driver| driver.machine_for_uri(uri.path)) {
            Ok(machine)
        } else {
            bail!("schema not found")
        }
    }
}

impl<E> Machine for Box<Machine<Error = E>> where E : ::std::error::Error + Send + 'static {
    type Error = Error;

    fn name(&self) -> &str {
        (**self).name()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        (**self).list_snapshots().chain_err(|| "list_snapshots")
    }

    fn stop(&mut self) -> Result<()> {
        (**self).stop().chain_err(|| "stop")
    }

    fn start(&mut self) -> Result<()> {
        (**self).start().chain_err(|| "start")
    }

    fn revert_to(&mut self, snapshot_name: &str) -> Result<()> {
        unimplemented!()
    }

    fn create_snapshot(&mut self, snapshot_name: &str) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    struct Nop;

    struct NopMachine(String);

    impl DriverFactory for Nop {
        fn machine_for_uri(&self, path: &str) -> Option<MachinePtr> {
            println!("me new {}", path);
            Some(Box::new(NopMachine(path.into())))
        }
    }

    impl Machine for NopMachine {
        type Error = Error;

        fn name(&self) -> &str {
            self.0.as_ref()
        }

        fn list_snapshots(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn stop(&mut self) -> Result<()> {
            unimplemented!()
        }

        fn start(&mut self) -> Result<()> {
            unimplemented!()
        }

        fn revert_to(&mut self, snapshot_name: &str) -> Result<()> {
            unimplemented!()
        }

        fn create_snapshot(&mut self, snapshot_name: &str) -> Result<()> {
            unimplemented!()
        }
    }

    #[test]
    fn test_repo() {
        let mut repo = Box::new(DriverRepo::default());

        repo.register("nop", Box::new(Nop));

        let m = repo.from_path("nop:smok1").unwrap();

        println!("m={}", m.name())
    }
}
