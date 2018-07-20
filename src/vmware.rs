use super::command;
use super::CommandRunner;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::rc::Rc;


error_chain! {

    foreign_links {
        Io(io::Error) #[doc = "Error during IO"];
    }

    links {
        Command(command::Error, command::ErrorKind);
    }

    errors {
        InvalidResponse(line : String)
        MissingSummary
    }

}

pub struct Driver<Cmd: CommandRunner> {
    inner: Rc<DriverImpl<Cmd>>,
}

impl<C: CommandRunner> Driver<C> {
    pub fn from_cmd(cmd: C) -> Self {
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
        Ok(self.command_runner
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
    fn run_list<Item: AsRef<OsStr>, List: IntoIterator<Item = Cow<'static, str>>>(
        &self,
        args: List,
    ) -> Result<command::Output> {
        let out = self.inner.run(&["list"])?;

        bail!("not ready")
    }

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
    type Error = Error;
    type Machine = MachineRef<Cmd>;

    fn list_running(&self) -> Result<Vec<MachineRef<Cmd>>> {
        let output = self.inner.run(&["list"])?;

        let mut it = output.into_iter();

        let n = match it.next() {
            Some(ref line) => if line.starts_with(VM_LIST_PREFIX) {
                let nstr : &str = line[VM_LIST_PREFIX.len()..].as_ref();
                nstr.parse::<usize>().chain_err(|| ErrorKind::InvalidResponse(line.to_string()))?
            } else {
                return Err(ErrorKind::InvalidResponse(line.to_string()).into())
            },
            None => return Err(ErrorKind::MissingSummary.into())
        };

        //let mut vms: Vec<Self::Machine> = Vec::with_capacity(n);

        it.take(n).map(|path| self.from_path(&path)).collect()
    }

    fn from_path(&self, path: &str) -> Result<MachineRef<Cmd>> {
        Ok(self.machine(path.to_string()))
    }
}

impl<Cmd: CommandRunner> super::Machine for MachineRef<Cmd> {
    type Error = Error;

    fn name(&self) -> &str {
        self.path.as_ref()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        let mut lines = self.driver_ref.run(&["listSnapshots", &self.path])?.into_iter();
        let summary : String = lines.next().chain_err(|| ErrorKind::MissingSummary)?;
        let n = if summary.starts_with(VM_SNAPSHOTS_PREFIX) {
            let s : &str = summary[VM_SNAPSHOTS_PREFIX.len() .. ].as_ref();
            s.parse::<usize>().chain_err(|| ErrorKind::InvalidResponse(summary.clone()))?
        }
        else {
            return Err(ErrorKind::InvalidResponse(summary.to_string()).into())
        };
        Ok(lines.collect())
    }

    fn stop(&mut self) -> Result<()> {
        let _ =  self.driver_ref.run(&["stop", &self.path, "hard"])?.into_iter();
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let _ =  self.driver_ref.run(&["start", &self.path, "nogui"])?.into_iter();
        Ok(())
    }

    fn revert_to<A : AsRef<str>>(&mut self, snapshot_name : A) -> Result<()> {
        let _ =  self.driver_ref.run(&["revertToSnapshot", &self.path, snapshot_name.as_ref()])?;
        self.start()
    }

    fn create_snapshot<A : AsRef<str>>(&mut self, snapshot_name: A) -> Result<()> {
        let _ =  self.driver_ref.run(&["snapshot", &self.path, snapshot_name.as_ref()])?;
        Ok(())
    }
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
