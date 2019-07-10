// Copyright 2019 Alan Somers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    ffi::CString,
    io::{self, Write},
    mem,
    ptr,
    slice
};
use super::TermInfo;
//use terminfo::Error::*;

pub(super) struct Database(ptr::NonNull<libc::DB>);

impl Database {
    pub(super) fn open() -> io::Result<Database> {
        let dbpath = CString::new("/usr/share/misc/termcap.db").unwrap();
        unsafe {
            let p = libc::dbopen(dbpath.as_ptr(), libc::O_RDONLY, 0,
                                 libc::DB_HASH, ptr::null());
            match ptr::NonNull::new(p) {
                Some(nnp) => Ok(Database(nnp)),
                None => Err(io::Error::last_os_error())
            }
        }
    }

    pub(super) fn get(&self, key: &libc::DBT) -> io::Result<&mut [u8]> {
        let mut data = mem::MaybeUninit::<libc::DBT>::uninit();
        unsafe {
            let r = ((*self.0.as_ptr()).get)(self.0.as_ptr(),
                                             key as *const libc::DBT,
                                             data.as_mut_ptr(), 0);
            match r {
                1 => Err(io::Error::from_raw_os_error(libc::ENOENT)),
                0 => {
                    let d = data.assume_init();
                    Ok(slice::from_raw_parts_mut::<u8>(d.data as *mut u8, d.size))
                 },
                _ => Err(io::Error::last_os_error()),
            }
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            ((*self.0.as_ptr()).close)(self.0.as_ptr());
        }
    }
}

pub(super) fn get_entry(name: &str) -> io::Result<Option<TermInfo>> {
    let db = Database::open()?;
    /*
     * Looking up the info is a 2-step process.  In the first step we
     * look up the full list of aliases from a single terminal name.  In
     * the second step we get the termcap info.
     */
    let termkey = libc::DBT {
        data: name.as_ptr() as *mut libc::c_void,
        size: name.len()
    };
    /* First get aliases */
    let aliases = db.get(&termkey)?;
    dbg!(std::str::from_utf8(aliases).unwrap());
    if aliases[0] != 2 {
        writeln!(io::stderr(), "Unexpected termcap entry").unwrap();
        return Ok(None);
    }

    /* Now get termcap data */
    let datakey = libc::DBT{
        data: aliases[1..].as_mut_ptr() as *mut libc::c_void,
        size: aliases.len() - 1
    };
    let data = db.get(&datakey)?;
    if data[0] != 0 {
        return Err(io::Error::from_raw_os_error(libc::EINVAL));
    }
    parse(std::str::from_utf8(&data[1..]).unwrap())
}

fn parse(entry: &str) -> io::Result<Option<TermInfo>> {
//fn parse(entry: &[u8]) -> io::Result<Option<TermInfo>> {
    //dbg!(&entry);
    let mut fields = entry.split(":");
    let names_str = fields.next().unwrap();
    let term_names: Vec<String> = names_str.split('|').map(|s| s.to_owned()).collect();
    for field in fields {
        if field == "\t" || field == "\0" {
            continue;
        }
        //dbg!(&field);
        if !field.starts_with("#") && field.contains("=") {
            // A string capability
            let mut parts = field.splitn(2, "=");
            println!("{} = {}", parts.next().unwrap(), parts.next().unwrap());
        } else if !field.starts_with("#") && field[1..].contains("#") {
            // A numeric capability
            let mut parts = field.splitn(2, "#");
            println!("{} # {}", parts.next().unwrap(), parts.next().unwrap());
        } else {
            // A boolean capability
            println!("{}", field);
        }
    }
    // TODO: create a TermInfo from dlslice
    Ok(None)
}
