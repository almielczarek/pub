use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, Utc};
use gotham::state::State;
use hyper::Uri;
use mime::{Mime, TEXT_HTML};
use mime_guess::guess_mime_type;
use percent_encoding::percent_decode;

use askama::Template;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Link {
    href: String,
    name: String,
    mtime: String,
}

#[derive(Template)]
#[template(path = "directory.html")]
struct DirectoryListing {
    current_path: String,
    links: Vec<Link>,
    breadcrumbs: Vec<Link>,
}

fn sanitize_path(path: &Path) -> PathBuf {
    let mut acc = PathBuf::new();

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

fn mtime(path: &Path) -> io::Result<String> {
    let metadata = fs::metadata(&path)?;
    let mtime = metadata.modified()?;
    let date_time: DateTime<Utc> = DateTime::from(mtime);

    Ok(format!("{}", date_time))
}

fn breadcrumbs(path: &Path) -> Vec<Link> {
    let sanitized = sanitize_path(path);
    let mut acc = PathBuf::new();
    let mut breadcrumbs = Vec::new();

    breadcrumbs.push(Link {
        href: String::from("/"),
        name: String::from("root"),
        mtime: String::from(""),
    });

    for component in sanitized.components() {
        acc.push(component);
        breadcrumbs.push(Link {
            href: String::from(acc.to_str().unwrap()),
            name: String::from(component.as_os_str().to_str().unwrap()),
            mtime: String::from(""),
        })
    }

    breadcrumbs
}

pub fn static_handler(state: State) -> (State, (Mime, Vec<u8>)) {
    let request_uri = state.borrow::<Uri>().path();
    let request_uri_decoded = percent_decode(request_uri.as_bytes())
        .decode_utf8()
        .unwrap()
        .to_string();

    let request_path = Path::new(&request_uri_decoded);

    let mut path = Path::new(".").to_path_buf();

    path.push(sanitize_path(request_path));

    if path.is_dir() {
        match directory_listing(&path) {
            Some(entries) => {
                let breadcrumbs = breadcrumbs(&path);
                let listing = DirectoryListing {
                    current_path: path.to_str().unwrap().to_string(),
                    links: entries,
                    breadcrumbs: breadcrumbs,
                };

                return (state, (TEXT_HTML, listing.render().unwrap().into_bytes()));
            }

            None => {
                return (
                    state,
                    (
                        TEXT_HTML,
                        format!("<h2>No file found at path: {}", &path.display()).into_bytes(),
                    ),
                );
            }
        }
    } else if path.is_file() {
        let mut buffer = Vec::new();
        let mut file = File::open(&path).unwrap();
        if let Err(e) = file.read_to_end(&mut buffer) {
            println!("Could not read file at {}", &path.display());
            println!("Error: {}", e);

            return (
                state,
                (
                    TEXT_HTML,
                    format!("<h2>Could not read file at path: {}", &path.display()).into_bytes(),
                ),
            );
        };

        let mime_type = guess_mime_type(&path);

        return (state, (mime_type, buffer));
    }

    (
        state,
        (
            TEXT_HTML,
            format!("<h2>No file found at path: {}", &path.display()).into_bytes(),
        ),
    )
}

fn directory_listing(path: &AsRef<Path>) -> Option<Vec<Link>> {
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut entries = entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = sanitize_path(&e.path());
                    let mtime = mtime(&path).unwrap(); // TODO: handle errors

                    Link {
                        href: String::from(path.to_str().unwrap()),
                        name: String::from(path.file_name().unwrap().to_str().unwrap()),
                        mtime: mtime,
                    }
                })
                .collect::<Vec<_>>();

            entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            Some(entries)
        }
        Err(_error) => None,
    }
}
