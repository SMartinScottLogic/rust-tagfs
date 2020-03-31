use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug)]
struct TagFSEntry {
    name: OsString,
    absolute: PathBuf,
    size: u64,
    tags: Vec<OsString>,
}

impl TagFSEntry {
    pub fn new(root: &str, entry: &walkdir::DirEntry, meta: &std::fs::Metadata) -> TagFSEntry {
        let components: Vec<_> = entry
            .path()
            .parent()
            .unwrap()
            .strip_prefix(root)
            .unwrap()
            .components()
            .map(|comp| comp.as_os_str().to_owned())
            .collect();
        let absolute = std::env::current_dir()
            .unwrap()
            .as_path()
            .join(entry.path())
            .canonicalize()
            .unwrap();
        TagFSEntry {
            name: entry.file_name().to_owned(),
            absolute,
            size: meta.size(),
            tags: components,
        }
    }
}

pub struct TagFS {
    root: String,
}

impl TagFS {
    pub fn new(root: &str) -> TagFS {
        let r = scan(root);
        debug!("{:?}", r);
        TagFS {
            root: root.to_string(),
        }
    }
}

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

fn process(root: &str, entry: &walkdir::DirEntry) -> Option<TagFSEntry> {
    let meta = match fs::metadata(entry.path()) {
        Ok(meta) => meta,
        _ => return None,
    };
    //info(&entry, &meta);
    if meta.is_file() {
        if let Some(_p) = entry.path().parent() {
            return Some(TagFSEntry::new(&root, &entry, &meta));
        }
    };
    None
}

fn scan(root: &str) -> Vec<TagFSEntry> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok().map(|entry| process(&root, &entry)).flatten())
        .collect()
}
