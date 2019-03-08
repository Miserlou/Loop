use std::fs::File;

pub trait StringFromTempfileStart {
    fn from_temp_start(tmpfile: &mut File) -> String;
}

impl StringFromTempfileStart for String {
    fn from_temp_start(tmpfile: &mut File) -> String {
        use std::io::{prelude::*, SeekFrom};

        let mut stdout = String::new();
        tmpfile.seek(SeekFrom::Start(0)).ok();
        tmpfile.read_to_string(&mut stdout).ok();
        stdout
    }
}
