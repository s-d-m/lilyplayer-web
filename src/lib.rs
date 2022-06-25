#[macro_use]
extern crate rocket;

use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use shuttle_service::ShuttleRocket;
use std::io::{ErrorKind, Read};

#[macro_use]
extern crate lazy_static;

// this function is copy/pasted from ureq. I use it here as the INTO_STRING_LIMIT is too small for my use case
fn default_read_to_end<R: Read + ?Sized>(
    r: &mut R,
    buf: &mut Vec<u8>,
) -> Result<usize, std::io::Error> {
    loop {
        let mut tmp_buf = [0u8; 32];

        match r.read(&mut tmp_buf) {
            Ok(nr_bytes) => {
                if nr_bytes == 0 {
                    return Ok(buf.len());
                }
                if buf.len() + nr_bytes > buf.capacity() {
                    buf.reserve((buf.len() * 3 / 2) + 5); // buf is full, need more space
                }

                buf.extend_from_slice(&tmp_buf[..nr_bytes]);
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

fn download_file(path: &str) -> Option<String> {
    let response = ureq::get(path).call();
    let body = match response {
        Ok(response) => {
            let mut buf: Vec<u8> = vec![];
            const INTO_STRING_LIMIT: usize = 100 * 1_024 * 1_024;

            let mut reader = response.into_reader().take((INTO_STRING_LIMIT + 1) as u64);

            match default_read_to_end(&mut reader, &mut buf) {
                Ok(_) => Some(String::from_utf8_lossy(buf.as_slice()).to_string()),
                Err(e) => {
                    println!("Error occurred {}", e);
                    None
                }
            }
        }
        Err(e) => {
            println!("Error occurred {}", e);
            None
        }
    };
    body
}

lazy_static! {
    static ref MAIN_FILE: Option<String> = {
        download_file(
            "https://raw.githubusercontent.com/s-d-m/lilyplayer-web/branch_with_assets/assets/lilyplayer.html",
        )
    };
    static ref WORKER_FILE: Option<String> = {
        download_file(
            "https://raw.githubusercontent.com/s-d-m/lilyplayer-web/branch_with_assets/assets/lilyplayer.worker.js",
        )
    };
}

#[derive(PartialEq, Eq)]
enum RequestedFile {
    MainLilyplayer,
    WorkerJs,
}

pub struct MyFile(RequestedFile);

impl<'r> Responder<'r, 'static> for MyFile {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let (content_type, data) = match self.0 {
            RequestedFile::MainLilyplayer => (ContentType::HTML, &*MAIN_FILE),
            RequestedFile::WorkerJs => (ContentType::JavaScript, &*WORKER_FILE),
            _ => return Err(rocket::http::Status::NotFound),
        };

        let mut response = Response::build();
        response.header(content_type);
        response.raw_header_adjoin("Cross-Origin-Opener-Policy", "same-origin");
        response.raw_header_adjoin("Cross-Origin-Embedder-Policy", "require-corp");
        match data {
            Some(x) => response.streamed_body(x.as_bytes()),
            None => &response,
        };
        Ok(response.finalize())
    }
}

#[get("/<path>")]
fn lilyplayer_files(path: &str) -> Option<MyFile> {
    if (path == "lilyplayer.html") || (path == "index.html") {
        Some(MyFile(RequestedFile::MainLilyplayer))
    } else if path == "lilyplayer.worker.js" {
        Some(MyFile(RequestedFile::WorkerJs))
    } else {
        None
    }
}

#[get("/")]
fn entry_point() -> Option<MyFile> {
    lilyplayer_files("lilyplayer.html")
}

#[shuttle_service::main]
async fn init() -> ShuttleRocket {
    let rocket = rocket::build()
        .mount("/", routes![lilyplayer_files])
        .mount("/", routes![entry_point]);

    Ok(rocket)
}