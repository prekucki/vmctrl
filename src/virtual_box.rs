
use super::command::{self, CommandRunner, Output};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::rc::Rc;

use nom::{self, is_alphanumeric};
use std::str;

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
    uuid : Option<String>,
}

impl<Cmd: CommandRunner> Driver<Cmd> {
    fn run_list<Item: AsRef<OsStr>, List: IntoIterator<Item = Cow<'static, str>>>(
        &self,
        args: List,
    ) -> Result<Output> {
        let out = self.inner.run(&["list"])?;

        bail!("not ready")
    }

    fn machine<IntoStr : Into<String>, IntoOpt : Into<String>>(&self, path: IntoStr, uuid : Option<IntoOpt>) -> MachineRef<Cmd> {
        MachineRef {
            driver_ref: self.inner.clone(),
            path: path.into(),
            uuid: uuid.map(|v| v.into())
        }
    }
}

impl<Cmd: CommandRunner> super::Driver for Driver<Cmd> {
    type Error = Error;
    type Machine = MachineRef<Cmd>;

    fn list_running(&self) -> Result<Vec<MachineRef<Cmd>>> {
        self.inner.run(&["list","runningvms"])?
            .into_iter()
            .map(|line| parse::vmslist_parse(line.as_ref())
                .map(|(name, uuid)| self.machine(name, Some(uuid))))
            .collect()
    }

    fn from_path(&self, path: &str) -> Result<MachineRef<Cmd>> {
        unimplemented!()
    }
}

impl<Cmd : CommandRunner> super::Machine for MachineRef<Cmd> {
    type Error = Error;

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn stop(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn start(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn revert_to<A: AsRef<str>>(&mut self, snapshot_name: A) -> Result<()> {
        unimplemented!()
    }

    fn create_snapshot<A: AsRef<str>>(&mut self, snapshot_name: A) -> Result<()> {
        unimplemented!()
    }
}


mod parse {
    use super::*;

    pub fn vmslist_parse<'a>(line: &'a str) -> Result<(&'a str, &'a str)> {
        let (_, r) = parse_vb_line(line.as_ref())
            .map_err(|_| ErrorKind::InvalidResponse(line.to_string()))?;

        Ok(r)
    }

    named!(parse_vb_line<(&str, &str)>,ws!(do_parse!(
        name : parse_word >>
        uuid : parse_uuid >>
        (name, uuid)
    )));

    fn to_s(i:Vec<u8>) -> String {
        String::from_utf8_lossy(&i).into_owned()
    }

    named!(parse_word<&str>, delimited!(
        tag!("\""),
        map_res!(
            escaped!(take_while1!(|c| c!= b'\"' && c!= b'\\'), '\\', one_of!("\"n\\")),
            str::from_utf8
        ),
        tag!("\"")
    ));

    named!(parse_uuid<&str>, map_res!(delimited!(
        char!('{'),
        take_while!(call!(|c| c != b'}')),
        char!('}')),
        str::from_utf8
    ));


    #[test]
    fn test_line() {
        let ins = "\"ubuntu-a ala\\n\\\"i psa\\\" ma kota\"";
        let (_, s) = parse_word(ins.as_ref()).unwrap();

        println!("{} ==> {}", ins, s);

        let ins2 = "{c777e3e8-b82e-40a4-bf3d-550f0f0da9e9}";
        let (_, s) = parse_uuid(ins2.as_ref()).unwrap();
        println!("{} ==> {}", ins2, s);

        let l = "\"ubuntu-a\" {c777e3e8-b82e-40a4-bf3d-550f0f0da9e9} ";
        let (_,b) = parse_vb_line(l.as_ref()).unwrap();

    }

}