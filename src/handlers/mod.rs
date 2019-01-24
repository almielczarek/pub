use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use gotham::state::State;
use hyper::Uri;
use mime::Mime;
use mime::{TEXT_HTML, TEXT_PLAIN};

use askama::Template;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Link {
    href: String,
    name: String,
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

fn breadcrumbs(path: &Path) -> Vec<Link> {
    let sanitized = sanitize_path(path);
    let mut acc = PathBuf::new();
    let mut breadcrumbs = Vec::new();

    breadcrumbs.push(Link {
        href: String::from("/"),
        name: String::from("root"),
    });

    for component in sanitized.components() {
        acc.push(component);
        breadcrumbs.push(Link {
            href: String::from(acc.to_str().unwrap()),
            name: String::from(component.as_os_str().to_str().unwrap()),
        })
    }

    breadcrumbs
}

pub fn static_handler(state: State) -> (State, (Mime, String)) {
    let request_path = Path::new(state.borrow::<Uri>().path());

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

                return (state, (TEXT_HTML, listing.render().unwrap()));
            }

            None => {
                return (
                    state,
                    (
                        TEXT_HTML,
                        format!("<h2>No file found at path: {}", &path.display()),
                    ),
                );
            }
        }
    } else if path.is_file() {
        let mut buffer = String::new();
        let mut file = File::open(&path).unwrap();
        file.read_to_string(&mut buffer);

        return (state, (TEXT_PLAIN, buffer))
    }

    (
        state,
        (
            TEXT_HTML,
            format!("<h2>No file found at path: {}", &path.display()),
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

                    Link {
                        href: String::from(path.to_str().unwrap()),
                        name: String::from(path.file_name().unwrap().to_str().unwrap()),
                    }
                })
                .collect::<Vec<_>>();

            entries.sort();

            Some(entries)
                },
        Err(_error) => None,
    }
}
