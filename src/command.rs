use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{Command, ExitStatus};
use std::str::{from_utf8, Utf8Error};
use super::error::*;

pub trait CommandRunner {
    fn run_with_output<C, I, S>(&self, cmd: C, args: I) -> Result<Output>
    where
        C: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
}

pub struct Output {
    inner: Vec<String>,
}

impl Output {
    fn new(mut v: Vec<String>) -> Self {
        let empty_last = match v.last() {
            Some(it) => it.len() == 0,
            None => false,
        };

        if empty_last {
            let _ = v.pop();
        }

        Output { inner: v }
    }
}

impl IntoIterator for Output {
    type Item = String;
    type IntoIter = <Vec<String> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.inner.into_iter()
    }
}

struct Local;

impl CommandRunner for Local {
    fn run_with_output<C, I, S>(&self, cmd: C, args: I) -> Result<Output>
    where
        C: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new(cmd).args(args).output()?;
        let status = output.status;
        if status.success() {
            let ret: ::std::result::Result<Vec<String>, Utf8Error> = output
                .stdout
                .split(|t| *t == '\n' as u8)
                .map(|it| from_utf8(it).map(|it| it.to_string()))
                .collect();

            return Ok(Output::new(ret?));
        }
        bail!(ErrorKind::Exec(
            status.code().unwrap_or(0i32),
            output.stderr.into(),
            output.stdout.into()
        ))
    }
}

struct Ssh {
    host: String,
}

fn escape_shell_chars<'a>(s: &'a OsStr) -> Cow<'a, OsStr> {
    let utf_str = s.to_string_lossy();
    let seq = utf_str.as_ref();

    let needs_esc = seq.chars().any(|ch| !ch.is_alphanumeric());
    if !needs_esc {
        return Cow::Borrowed(s);
    }

    let mut result: String = seq.chars().fold(String::from("'"), |mut s, ch| {
        if ch == '\'' {
            s.push_str("'\\''")
        } else {
            s.push(ch)
        }
        s
    });

    result.push('\'');

    Cow::Owned(result.into())
}

impl CommandRunner for Ssh {
    fn run_with_output<C, I, S>(&self, cmd: C, args: I) -> Result<Output>
    where
        C: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut shell_command = OsString::default();
        shell_command.push(escape_shell_chars(cmd.as_ref()));
        for arg in args.into_iter() {
            shell_command.push(" ");
            shell_command.push(escape_shell_chars(arg.as_ref()))
        }
        let host: &OsStr = self.host.as_ref();
        let output = Command::new("ssh")
            .args(&[host, shell_command.as_ref()])
            .output()?;
        let status = output.status;
        if status.success() {
            let ret: ::std::result::Result<Vec<String>, Utf8Error> = output
                .stdout
                .split(|t| *t == '\n' as u8)
                .map(|it| from_utf8(it).map(|it| it.to_string()))
                .collect();
            return Ok(Output::new(ret?));
        }
        bail!(ErrorKind::Exec(
            status.code().unwrap_or(0i32),
            output.stderr.into(),
            output.stdout.into()
        ))
    }
}

pub fn local() -> impl CommandRunner {
    Local
}

pub fn ssh<T: Into<String>>(host: T) -> impl CommandRunner {
    Ssh { host: host.into() }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_escape_shell_chars() {
        let a1: &OsStr = "ala".as_ref();
        let a2: &OsStr = "'ala ma kota'".as_ref();
        assert_eq!(escape_shell_chars(a1), a1);
        assert_eq!(escape_shell_chars("ala ma kota".as_ref()), a2)
    }

}
