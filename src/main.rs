mod tagfs;

use chrono::Local;
use std::{env, io};
/*
use std::os::unix::fs::MetadataExt;
use std::{env, fs, io};
use walkdir::WalkDir;
use std::path::PathBuf;
*/

use crate::tagfs::TagFS;

#[macro_use]
extern crate log;

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!(
            "{} {} {} - {}",
            Local::now().format("%Y-%m-%dT%H:%M:%S%z"),
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOGGER: ConsoleLogger = ConsoleLogger;

fn main() -> io::Result<()> {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    let args: Vec<String> = env::args().collect();

    let tag_fs = TagFS::new(&args[1]);

    /*
    for a in env::args().skip(1) {
        let r = scan(&a);
        debug!("{:?}", r);
    }
    */

    debug!("Hi");
    trace!("Hello, world!");
    info!("bye");
    Ok(())
}
