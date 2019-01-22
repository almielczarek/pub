use std::fs::{self, DirEntry, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use gotham::state::State;
use hyper::Uri;
use mime::Mime;
use mime::{TEXT_HTML, TEXT_PLAIN};

fn sanitize_path(path: &Path) -> PathBuf {
    path.components()
        .fold(PathBuf::new(), |mut acc, component| match component {
            Component::Normal(x) => {
                acc.push(x);
                acc
            }
            _ => acc,
        })
}

pub fn static_handler(state: State) -> (State, (Mime, String)) {
    let request_path = Path::new(state.borrow::<Uri>().path());

    let mut path = Path::new(".").to_path_buf();

    path.push(sanitize_path(request_path));

    let resp = match fs::metadata(&path) {
        Ok(meta) => {
            if meta.is_dir() {
                (TEXT_HTML, directory_listing(&path))
            } else {
                let mut buffer = String::new();
                let mut file = File::open(&path).unwrap();
                file.read_to_string(&mut buffer);

                (TEXT_PLAIN, buffer)
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            (
                TEXT_HTML,
                format!("<h2>No file found at path: {}", &path.display()),
            )
        }
    };

    (state, resp)
}

fn render_link(entry: DirEntry) -> String {
    format!(
        "<li><a href=\"/{}\">{}</a></li>",
        sanitize_path(&entry.path()).display(),
        entry.path().file_name().unwrap().to_str().unwrap()
    )
}

fn directory_listing(path: &AsRef<Path>) -> String {
    let mut entries = fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| render_link(e))
        .collect::<Vec<_>>();

    entries.sort();

    format!("<ul>{}</ul>", entries.join(""))
}
