use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, Utc};
use mime::Mime;
use mime_guess::guess_mime_type;

fn path_to_string(path: &Path) -> String {
    match path.to_str() {
        Some(s) => s.to_string(),
        None => String::new(),
    }
}

fn os_str_to_string(os_str: &OsStr) -> String {
    match os_str.to_str() {
        Some(s) => s.to_string(),
        None => String::new(),
    }
}

fn breadcrumbs(path: &Path) -> Vec<(String, String)> {
    let sanitized = sanitize_path(&path);
    let mut acc = PathBuf::new();
    let mut breadcrumbs = Vec::new();

    breadcrumbs.push((String::from("/"), String::from("root")));

    for component in sanitized.components() {
        match component {
            Component::Normal(x) => {
                acc.push(x);

                breadcrumbs.push((
                    path_to_string(&acc),
                    os_str_to_string(component.as_os_str()),
                ))
            }
            _ => (),
        }
    }

    breadcrumbs
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Entry {
    pub href: String,
    pub name: String,
    pub mtime: String,
    pub size: u64, // In bytes
}

impl Entry {
    fn new(fs_entry: fs::DirEntry) -> io::Result<Self> {
        let meta = fs::metadata(&fs_entry.path())?;
        let mtime = meta.modified()?;
        let mtime_datetime: DateTime<Utc> = DateTime::from(mtime);

        Ok(Entry {
            href: path_to_string(&fs_entry.path()),
            name: String::from(fs_entry.file_name().to_string_lossy()),
            mtime: format!("{}", mtime_datetime),
            size: meta.len(),
        })
    }
}

#[derive(Debug)]
pub enum Static {
    Directory {
        current_path: String,
        breadcrumbs: Vec<(String, String)>,
        entries: Vec<Entry>,
    },
    File {
        mime_type: Mime,
        buffer: Vec<u8>,
    },
}

fn sanitize_path(path: &Path) -> PathBuf {
    let mut acc = Path::new(".").to_path_buf();

    for component in path.components() {
        match component {
            Component::Normal(x) => {
                acc.push(x);
            }
            _ => (),
        }
    }

    acc
}

use Static::*;

impl Static {
    pub fn new(path: &Path) -> io::Result<Self> {
        let path = sanitize_path(path);

        if path.is_file() {
            let mut buffer = Vec::new();
            let mut file = fs::File::open(&path)?;
            file.read_to_end(&mut buffer)?;

            Ok(File {
                mime_type: guess_mime_type(&path),
                buffer: buffer,
            })
        } else {
            let fs_entries = fs::read_dir(&path)?;

            let mut entries = fs_entries
                .filter_map(|e| e.ok())
                .map(|e| Entry::new(e))
                .filter_map(|e| e.ok())
                .collect::<Vec<Entry>>();

            entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            Ok(Directory {
                current_path: path_to_string(&path),
                breadcrumbs: breadcrumbs(&path),
                entries: entries,
            })
        }
    }
}
