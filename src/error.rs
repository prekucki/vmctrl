use std::{fmt, io, str};

error_chain! {

    foreign_links {
        Io(io::Error) #[doc = "Error during IO"];
        UTF8(str::Utf8Error);
    }


    errors {
        InvalidResponse(line : String)
        MissingSummary
        Exec(code : i32, stderr : ProcessOutput, stdout : ProcessOutput) {
            description("shell command exec failed")
            display("Error code {}", code)
        }
    }
}

pub struct ProcessOutput {
    c: Vec<u8>,
}

impl From<Vec<u8>> for ProcessOutput {
    fn from(src: Vec<u8>) -> Self {
        ProcessOutput { c: src }
    }
}

impl fmt::Debug for ProcessOutput {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        if let Ok(s) = str::from_utf8(self.c.as_ref()) {
            fmt::Debug::fmt(s, f)
        } else {
            fmt::Debug::fmt(&self.c, f)
        }
    }
}
