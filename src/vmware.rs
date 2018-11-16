use super::uri::DriverFactory;
use super::{command, CommandRunner, FromCommandRunner, Machine};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::rc::Rc;

use super::error::*;

pub struct Driver<Cmd: CommandRunner> {
    inner: Rc<DriverImpl<Cmd>>,
}

pub struct Factory<C: CommandRunner> {
    marker: PhantomData<C>,
}

#[inline]
pub fn factory<C: CommandRunner>() -> Factory<C> {
    Factory {
        marker: PhantomData,
    }
}

impl<C: CommandRunner> command::FromCommandRunner for Factory<C> {
    type Command = C;
    type Output = Driver<C>;

    fn from_cmd(&self, cmd: Self::Command) -> Self::Output {
        Driver {
            inner: Rc::new(DriverImpl {
                command_runner: cmd,
                vmrun_command: Cow::Borrowed("vmrun".as_ref()),
            }),
        }
    }
}

impl<C: CommandRunner> DriverImpl<C> {
    fn run<I, S>(&self, args: I) -> Result<command::Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Ok(self
            .command_runner
            .run_with_output(&self.vmrun_command, args)?)
    }
}

struct DriverImpl<Cmd: CommandRunner> {
    command_runner: Cmd,
    vmrun_command: Cow<'static, OsStr>,
}

pub struct MachineRef<Cmd: CommandRunner> {
    driver_ref: Rc<DriverImpl<Cmd>>,
    path: String,
}

impl<Cmd: CommandRunner> Driver<Cmd> {
    fn machine(&self, path: String) -> MachineRef<Cmd> {
        MachineRef {
            driver_ref: self.inner.clone(),
            path,
        }
    }
}

impl<C: CommandRunner + Default> Default for Driver<C> {
    fn default() -> Self {
        Driver {
            inner: Rc::new(DriverImpl {
                command_runner: C::default(),
                vmrun_command: Cow::Borrowed("vmrun".as_ref()),
            }),
        }
    }
}

const VM_LIST_PREFIX: &str = "Total running VMs: ";
const VM_SNAPSHOTS_PREFIX: &str = "Total snapshots: ";

impl<Cmd: CommandRunner> super::Driver for Driver<Cmd> {
    type Machine = MachineRef<Cmd>;

    fn list_running(&self) -> Result<Vec<MachineRef<Cmd>>> {
        let output = self.inner.run(&["list"])?;

        let mut it = output.into_iter();

        let n = match it.next() {
            Some(ref line) => if line.starts_with(VM_LIST_PREFIX) {
                let nstr: &str = line[VM_LIST_PREFIX.len()..].as_ref();
                nstr.parse::<usize>()
                    .chain_err(|| ErrorKind::InvalidResponse(line.to_string()))?
            } else {
                return Err(ErrorKind::InvalidResponse(line.to_string()).into());
            },
            None => return Err(ErrorKind::MissingSummary.into()),
        };

        //let mut vms: Vec<Self::Machine> = Vec::with_capacity(n);

        it.take(n).map(|path| self.from_path(&path)).collect()
    }

    fn from_path(&self, path: &str) -> Result<MachineRef<Cmd>> {
        Ok(self.machine(path.to_string()))
    }
}

impl<Cmd: CommandRunner> super::Machine for MachineRef<Cmd> {
    fn name(&self) -> &str {
        self.path.as_ref()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        let mut lines = self
            .driver_ref
            .run(&["listSnapshots", &self.path])?
            .into_iter();
        let summary: String = lines.next().chain_err(|| ErrorKind::MissingSummary)?;
        let _n = if summary.starts_with(VM_SNAPSHOTS_PREFIX) {
            let s: &str = summary[VM_SNAPSHOTS_PREFIX.len()..].as_ref();
            s.parse::<usize>()
                .chain_err(|| ErrorKind::InvalidResponse(summary.clone()))?
        } else {
            return Err(ErrorKind::InvalidResponse(summary.to_string()).into());
        };
        Ok(lines.collect())
    }

    fn stop(&mut self) -> Result<()> {
        let _ = self
            .driver_ref
            .run(&["stop", &self.path, "hard"])?
            .into_iter();
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let _ = self
            .driver_ref
            .run(&["start", &self.path, "nogui"])?
            .into_iter();
        Ok(())
    }

    fn revert_to(&mut self, snapshot_name: &str) -> Result<()> {
        let _ = self
            .driver_ref
            .run(&["revertToSnapshot", &self.path, snapshot_name.as_ref()])?;
        self.start()
    }

    fn create_snapshot(&mut self, snapshot_name: &str) -> Result<()> {
        let _ = self
            .driver_ref
            .run(&["snapshot", &self.path, snapshot_name.as_ref()])?;
        Ok(())
    }
}

impl<Cmd: CommandRunner + 'static> DriverFactory for Driver<Cmd> {
    fn machine_for_uri(&self, uri: &str) -> Option<Box<Machine>> {
        Some(Box::new(self.machine(uri.into())))
    }
}

pub fn local_driver() -> Box<DriverFactory> {
    Box::new(factory().from_cmd(command::local()))
}

pub fn remote_driver() -> Box<DriverFactory> {

    factory().into()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cow() {
        let c: Cow<'static, str> = "vmrun".into();

        println!("test {}", &c);
    }

}
