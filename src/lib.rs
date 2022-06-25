#[macro_use]
extern crate rocket;

use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use shuttle_service::ShuttleRocket;

#[macro_use]
extern crate lazy_static;

fn download_files(path: &str) -> Option<String> {
    let response = ureq::get(path).call();
    let body = match response {
        Ok(response) => response.into_string(),
        Err(E) => {
            println!("Error occurred {}", E);
            return None;
        }
    };
    match body {
        Ok(string_value) => Some(string_value),
        Err(E) => {
            println!("Error occurred {}", E);
            None
        }
    }
}

lazy_static! {
    static ref main_file: Option<String> = {
        let first_file = download_files("https://");
        first_file
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
            (ContentType::HTML, &*main_file)
        } else if self.0 == RequestedFile::WorkerJs {
            (ContentType::HTML, &*main_file)
        //            (ContentType::JavaScript, Some(String::from(worker_js)))
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

#[get("/myfile/<path>")]
fn myfile(path: &str) -> Option<MyFile> {
    if (path == "lilyplayer.html") || (path == "index.html") {
        Some(MyFile(RequestedFile::MainLilyplayer))
    } else if path == "lilyplayer.worker.js" {
        Some(MyFile(RequestedFile::WorkerJs))
    } else {
        None
    }
}

#[shuttle_service::main]
async fn init() -> ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![myfile]);

    Ok(rocket)
}