#![feature(read_buf)]
#[macro_use]
extern crate rocket;

use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use shuttle_service::ShuttleRocket;
use std::io::{ErrorKind, Read, ReadBuf};

#[macro_use]
extern crate lazy_static;

// this function is copy/pasted from ureq. I use it here as the INTO_STRING_LIMIT is too small for my use case
fn default_read_to_end<R: Read + ?Sized>(
    r: &mut R,
    buf: &mut Vec<u8>,
) -> Result<usize, std::io::Error> {
    let start_len = buf.len();
    let start_cap = buf.capacity();

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(32); // buf is full, need more space
        }

        let mut read_buf = ReadBuf::uninit(buf.spare_capacity_mut());

        // SAFETY: These bytes were initialized but not filled in the previous loop
        unsafe {
            read_buf.assume_init(initialized);
        }

        match r.read_buf(&mut read_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }

        if read_buf.filled_len() == 0 {
            return Ok(buf.len() - start_len);
        }

        // store how much was initialized but not filled
        initialized = read_buf.initialized_len() - read_buf.filled_len();
        let new_len = read_buf.filled_len() + buf.len();

        // SAFETY: ReadBuf's invariants mean this much memory is init
        unsafe {
            buf.set_len(new_len);
        }

        if buf.len() == buf.capacity() && buf.capacity() == start_cap {
            // The buffer might be an exact fit. Let's read into a probe buffer
            // and see if it returns `Ok(0)`. If so, we've avoided an
            // unnecessary doubling of the capacity. But if not, append the
            // probe buffer to the primary buffer and let its capacity grow.
            let mut probe = [0u8; 32];

            loop {
                match r.read(&mut probe) {
                    Ok(0) => return Ok(buf.len() - start_len),
                    Ok(n) => {
                        buf.extend_from_slice(&probe[..n]);
                        break;
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
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
            "https://raw.githubusercontent.com/s-d-m/lilyplayer-web/master/assets/lilyplayer.html",
        )
    };
    static ref WORKER_FILE: Option<String> = {
        download_file(
            "https://raw.githubusercontent.com/s-d-m/lilyplayer-web/master/assets/lilyplayer.worker.js",
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
        let (content_type, data) = if self.0 == RequestedFile::MainLilyplayer {
            (ContentType::HTML, &*MAIN_FILE)
        } else if self.0 == RequestedFile::WorkerJs {
            (ContentType::JavaScript, &*WORKER_FILE)
        } else {
            return Err(rocket::http::Status::NotFound);
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