use std::io::Cursor;
use std::path::Path;

use askama::Template;
use chrono::Local;
use gotham::state::State;
use hyper::{Body, Response, StatusCode, Uri};
use mime::{Mime, TEXT_HTML};
use percent_encoding::percent_decode;

use crate::pub_static::{Entry, Static};

#[derive(Template)]
#[template(path = "directory.html")]
struct DirectoryTemplate {
    current_path: String,
    breadcrumbs: Vec<(String, String)>,
    entries: Vec<Entry>,
}

pub fn static_handler(state: State) -> (State, (Mime, Vec<u8>)) {
    let request_uri = state.borrow::<Uri>().path();
    let request_uri_decoded = percent_decode(request_uri.as_bytes())
        .decode_utf8()
        .unwrap()
        .to_string();

    let request_path = Path::new(&request_uri_decoded);

    let time = Local::now();
    println!("[{}] GET {}", time, request_path.display());

    let response = match Static::new(&request_path) {
        Ok(s) => match s {
            Static::File { buffer, mime_type } => (mime_type, buffer),
            Static::Directory {
                current_path,
                breadcrumbs,
                entries,
            } => {
                let template = DirectoryTemplate {
                    current_path: current_path,
                    breadcrumbs: breadcrumbs,
                    entries: entries,
                };

                (TEXT_HTML, template.render().unwrap().into_bytes())
            }
        },
        Err(e) => {
            match request_path.to_str() {
                Some("/favicon.ico") => (), // We do this here so we're not polluting the error output
                _ => {
                    eprintln!("Error: {}", e);
                }
            }
            (TEXT_HTML, String::from("bad").into_bytes())
        }
    };

    (state, response)
}

pub fn zip_archive_handler(state: State) -> (State, hyper::Response<Body>) {
    let request_uri = state.borrow::<Uri>().path();
    let request_uri_decoded = percent_decode(request_uri.as_bytes())
        .decode_utf8()
        .unwrap()
        .to_string();

    let time = Local::now();
    println!("[{}] GET {}", time, request_uri_decoded);

    let request_path = Path::new(&request_uri_decoded)
        .strip_prefix("/archive")
        .unwrap();

    let request_path = Path::new(".").join(request_path);

    let buffer = Vec::new();
    let w = Cursor::new(buffer);
    let w = zip_dir::zip_dir(&request_path, w, None).unwrap();

    let mut response = Response::builder();

    response
        .header("Content-Type", "application/zip")
        .status(StatusCode::OK);

    (state, response.body(w.into_inner().into()).unwrap())
}
