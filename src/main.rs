use chrono::Local;
use std::os::unix::fs::MetadataExt;
use std::{fs, io};
use walkdir::WalkDir;

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

fn info(entry: &walkdir::DirEntry, meta: &std::fs::Metadata) {
    let dev_id = meta.dev();
    let inode = meta.ino();
    println!(
        "{} {} {} {:o} {:?} {} {} (@ {})",
        dev_id,
        inode,
        entry.path().display(),
        meta.mode(),
        meta.is_dir(),
        meta.is_file(),
        meta.size(),
        std::env::current_dir()
            .unwrap()
            .as_path()
            .join(entry.path())
            .canonicalize()
            .unwrap()
            .display()
    );
}

fn scan(root: &str) -> Result<(), std::io::Error> {
    let r: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| {
            entry
                .ok()
                .map(|entry| {
                    let meta = match fs::metadata(entry.path()) {
                        Ok(meta) => meta,
                        _ => return None,
                    };
                    info(&entry, &meta);
                    if meta.is_file() {
                        let path = entry.into_path();
                        if let Some(p) = path.parent() {
                            let components: Vec<_> = p
                                .strip_prefix(root)
                                .unwrap()
                                .components()
                                .map(|comp| comp.as_os_str().to_owned())
                                .collect();
                            return Some((path, components));
                        }
                    };
                    None
                })
                .flatten()
        })
        .collect();
    println!("{:?}", r);
    Ok(())
}

fn main() -> io::Result<()> {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    scan(&String::from("src"))?;

    debug!("Hi");
    trace!("Hello, world!");
    info!("bye");
    Ok(())
}
