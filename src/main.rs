mod libc_wrapper;
mod tagfs;

use chrono::Local;
use std::ffi::OsStr;
use std::{env, io};

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

    debug!("Hi");
    trace!("Hello, world!");
    info!("bye");

    let fuse_args: Vec<&OsStr> = vec![OsStr::new("-o"), OsStr::new("auto_unmount")];

    fuse_mt::mount(fuse_mt::FuseMT::new(tag_fs, 1), &args[2], &fuse_args).unwrap();
    Ok(())
}
