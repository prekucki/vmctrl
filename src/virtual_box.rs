use super::command::{self, CommandRunner, Output};
use super::uri::DriverFactory;
use super::Machine;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::rc::Rc;

use regex::Regex;
use std::str;
use super::error::*;

pub struct Driver<Cmd: CommandRunner> {
    inner: Rc<DriverImpl<Cmd>>,
}

struct DriverImpl<Cmd: CommandRunner> {
    command_runner: Cmd,
    manage_command: Cow<'static, OsStr>,
}

impl<C: CommandRunner> Driver<C> {
    pub fn from_cmd(cmd: C) -> Self {
        Driver {
            inner: Rc::new(DriverImpl {
                command_runner: cmd,
                manage_command: Cow::Borrowed("vboxmanage".as_ref()),
            }),
        }
    }
}

impl<C: CommandRunner> DriverImpl<C> {
    fn run<I, S>(&self, args: I) -> Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Ok(self.command_runner
            .run_with_output(&self.manage_command, args)?)
    }
}

pub struct MachineRef<Cmd: CommandRunner> {
    driver_ref: Rc<DriverImpl<Cmd>>,
    path: String,
    uuid: Option<String>,
}

impl<Cmd: CommandRunner> Driver<Cmd> {
    fn run_list<Item: AsRef<OsStr>, List: IntoIterator<Item = Cow<'static, str>>>(
        &self,
        args: List,
    ) -> Result<Output> {
        let out = self.inner.run(&["list"])?;

        bail!("not ready")
    }

    fn machine<IntoStr: Into<String>>(
        &self,
        path: IntoStr,
        uuid: Option<String>,
    ) -> MachineRef<Cmd> {
        MachineRef {
            driver_ref: self.inner.clone(),
            path: path.into(),
            uuid: uuid.map(|v| v.into()),
        }
    }
}

impl<Cmd: CommandRunner> super::Driver for Driver<Cmd> {
    type Error = Error;
    type Machine = MachineRef<Cmd>;

    fn list_running(&self) -> Result<Vec<MachineRef<Cmd>>> {
        self.inner
            .run(&["list", "runningvms"])?
            .into_iter()
            .map(|line| {
                vmslist_parse(line.as_ref())
                    .map(|(name, uuid)| self.machine(name, Some(uuid.into())))
            })
            .collect()
    }

    fn from_path(&self, path: &str) -> Result<MachineRef<Cmd>> {
        Ok(self.machine(path, None))
    }
}

fn vmslist_parse(line: &str) -> Result<(&str, &str)> {
    lazy_static! {
        static ref RE: Regex = Regex::new("\"([^\"]*)\"\\s+(\\{[a-zA-Z0-9-]*\\})").unwrap();
    }

    if let Some(caps) = RE.captures(line) {
        if let (Some(name), Some(uuid)) = (caps.get(1), caps.get(2)) {
            return Ok((name.as_str(), uuid.as_str()));
        }
    }

    bail!("invalid")
}

pub fn init() {

}

#[test]
fn test_vmslist_parse() {
    let (a, b) = vmslist_parse("\"ubuntu-a\" {c777e3e8-b82e-40a4-bf3d-550f0f0da9e9}").unwrap();

    assert_eq!(a, "ubuntu-a");
    assert_eq!(b, "{c777e3e8-b82e-40a4-bf3d-550f0f0da9e9}");
}

impl<T: CommandRunner> MachineRef<T> {
    fn vmid(&self) -> &str {
        self.uuid.as_ref().unwrap_or(&self.path)
    }
}

impl<Cmd: CommandRunner + 'static> DriverFactory for Driver<Cmd> {
    fn machine_for_uri(&self, uri: &str) -> Option<Box<Machine<Error=Error>>> {
        Some(Box::new(self.machine(uri, None)))
    }
}


pub fn local_driver() -> Box<DriverFactory> {
    Box::new(Driver::from_cmd(command::local()))
}


impl<Cmd: CommandRunner> super::Machine for MachineRef<Cmd> {
    type Error = Error;

    fn name(&self) -> &str {
        self.path.as_ref()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        let output = self.driver_ref
            .run(&["snapshot", self.vmid(), "list", "--machinereadable"]);
        let prop_re = Regex::new("^([a-zA-Z0-9\\-]+)=\"([^\"]*)\"$").unwrap();
        let mut res = Vec::new();

        for line in output? {
            let (k, v) = match prop_re.captures(&line) {
                Some(cap) => match (cap.get(1), cap.get(2)) {
                    (Some(k), Some(v)) => (k.as_str(), v.as_str()),
                    _ => bail!(ErrorKind::InvalidResponse(line.clone())),
                },
                _ => bail!(ErrorKind::InvalidResponse(line.clone())),
            };
            if k.starts_with("SnapshotName") {
                res.push(v.into())
            }
        }
        Ok(res)
    }

    fn stop(&mut self) -> Result<()> {
        let _ = self.driver_ref.run(&["controlvm", self.vmid(), "poweroff"]);
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let _ = self.driver_ref
            .run(&["startvm", self.vmid(), "--type", "headless"])?;
        Ok(())
    }

    fn revert_to(&mut self, snapshot_name: &str) -> Result<()> {
        let _ = self.driver_ref
            .run(&["snapshot", self.vmid(), "restore", snapshot_name])?;
        Ok(())
    }

    fn create_snapshot(&mut self, snapshot_name: &str) -> Result<()> {
        let _ = self.driver_ref
            .run(&["snapshot", self.vmid(), "take", snapshot_name])?;
        Ok(())
    }
}
