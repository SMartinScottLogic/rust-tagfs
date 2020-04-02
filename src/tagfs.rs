use fuse_mt::{
    DirectoryEntry, FileAttr, FileType, FilesystemMT, RequestInfo, ResultEmpty, ResultEntry,
    ResultOpen, ResultReaddir, Statfs,
};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component::Normal, Path, PathBuf};
use time::Timespec;
use walkdir::WalkDir;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

#[derive(Debug)]
struct TagFSEntry {
    name: OsString,
    absolute: PathBuf,
    size: u64,
    tags: HashSet<OsString>,
}

impl TagFSEntry {
    pub fn new(root: &str, entry: &walkdir::DirEntry, meta: &std::fs::Metadata) -> TagFSEntry {
        let components: HashSet<_> = entry
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
    tags: HashSet<OsString>,
    entries: Vec<TagFSEntry>,
}

impl TagFS {
    pub fn new(root: &str) -> TagFS {
        let entries = scan(root);
        debug!("{:?}", entries);
        TagFS {
            root: root.to_string(),
            tags: entries
                .iter()
                .flat_map(|tag_entry| tag_entry.tags.clone())
                .collect(),
            entries,
        }
    }

    fn mode_to_filetype(mode: libc::mode_t) -> FileType {
        match mode & libc::S_IFMT {
            libc::S_IFDIR => FileType::Directory,
            libc::S_IFREG => FileType::RegularFile,
            libc::S_IFLNK => FileType::Symlink,
            libc::S_IFBLK => FileType::BlockDevice,
            libc::S_IFCHR => FileType::CharDevice,
            libc::S_IFIFO => FileType::NamedPipe,
            libc::S_IFSOCK => FileType::Socket,
            _ => {
                panic!("unknown file type");
            }
        }
    }

    fn stat_to_fuse() -> FileAttr {
        FileAttr {
            size: 0,
            blocks: 0,
            atime: Timespec { sec: 0, nsec: 0 },
            mtime: Timespec { sec: 0, nsec: 0 },
            ctime: Timespec { sec: 0, nsec: 0 },
            crtime: Timespec { sec: 0, nsec: 0 },
            kind: FileType::Directory,
            perm: 0o0755,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        }
    }
    /*
        fn stat_to_fuse(stat: libc::stat) -> FileAttr {
            // st_mode encodes both the kind and the permissions
            let kind = TagFS::mode_to_filetype(stat.st_mode);
            let perm = (stat.st_mode & 0o7777) as u16;

            FileAttr {
                size: stat.st_size as u64,
                blocks: stat.st_blocks as u64,
                atime: Timespec { sec: stat.st_atime as i64, nsec: stat.st_atime_nsec as i32 },
                mtime: Timespec { sec: stat.st_mtime as i64, nsec: stat.st_mtime_nsec as i32 },
                ctime: Timespec { sec: stat.st_ctime as i64, nsec: stat.st_ctime_nsec as i32 },
                crtime: Timespec { sec: 0, nsec: 0 },
                kind,
                perm,
                nlink: stat.st_nlink as u32,
                uid: stat.st_uid,
                gid: stat.st_gid,
                rdev: stat.st_rdev as u32,
                flags: 0,
            }
        }
    */
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

impl FilesystemMT for TagFS {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        debug!("init");
        Ok(())
    }

    fn destroy(&self, _req: RequestInfo) {
        debug!("destroy");
    }

    fn getattr(&self, _req: RequestInfo, path: &Path, fh: Option<u64>) -> ResultEntry {
        debug!("getattr: {:?} {:?}", path, fh);

        Ok((TTL, TagFS::stat_to_fuse()))
        /*

        if let Some(fh) = fh {
            match libc_wrappers::fstat(fh) {
                Ok(stat) => Ok((TTL, stat_to_fuse(stat))),
                Err(e) => Err(e)
            }
        } else {
            match self.stat_real(path) {
                Ok(attr) => Ok((TTL, attr)),
                Err(e) => Err(e.raw_os_error().unwrap())
            }
        }
        */
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        debug!("opendir: {:?} (flags = {:#o})", path, _flags);
        //let real = self.real_path(path);
        Ok((0, 0))
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        debug!("readdir: {:?}", path);
        let cur_tags: HashSet<OsString> = path
            .components()
            .filter_map(|c| match c {
                Normal(t) => Some(t.to_os_string()),
                _ => None,
            })
            .collect();
        debug!("components: {:?}", cur_tags);
        let mut entries: Vec<DirectoryEntry> = vec![];
        for tag in &self.tags {
            if !cur_tags.contains(tag) {
                entries.push(DirectoryEntry {
                    name: tag.to_os_string(),
                    kind: FileType::Directory,
                });
            }
        }
        for entry in &self.entries {
            if entry.tags.is_superset(&cur_tags) {
                debug!("match {:?}", entry);
                entries.push(DirectoryEntry {
                    name: entry.name.to_os_string(),
                    kind: FileType::RegularFile,
                });
            }
        }
        /*
        let real = self.real_path(path);
        // Consider using libc::readdir to prevent need for always stat-ing entries
        let iter = match fs::read_dir(&real) {
            Ok(iter) => iter,
            Err(e) => return Err(e.raw_os_error().unwrap_or(ENOENT))
        };
        for entry in iter {
            match entry {
                Ok(entry) => {
                    let real_path = entry.path();
                    debug!("readdir: {:?} {:?}", real, real_path);
                    let stat = match libc_wrapper::lstat(real_path.clone()) {
                        Ok(stat) => stat,
                        Err(e) => return Err(e.raw_os_error().unwrap_or(ENOENT))
                    };
                    let filetype = DecoFS::stat_to_filetype(&stat);

                    entries.push(DirectoryEntry {
                        name: real_path.file_name().unwrap().to_os_string(),
                        kind: filetype,
                    });
                },
                Err(e) => {
                    error!("readdir: {:?}: {}", path, e);
                    return Err(e.raw_os_error().unwrap_or(ENOENT));
                }
            }
        }
        */
        info!("entries: {:?}", entries);
        Ok(entries)
    }
}
