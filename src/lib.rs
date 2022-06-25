#[macro_use]
extern crate rocket;

use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;

use shuttle_service::ShuttleRocket;


#[derive(PartialEq, Eq)]
enum RequestedFile {
    MainLilyplayer,
    WorkerJs
}

pub struct MyFile(RequestedFile);

impl<'r> Responder<'r, 'static> for MyFile {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let main_html = include_str!("lilyplayer.html");
        let worker_js = include_str!("lilyplayer.worker.js");


        let (content_type, data) =
            if self.0 == RequestedFile::MainLilyplayer {
                (ContentType::HTML, main_html)
            } else if self.0 == RequestedFile::WorkerJs {
                (ContentType::JavaScript, worker_js)
            } else {
                return Err(rocket::http::Status::NotFound);
            };

        let mut response = Response::build();
        response.header(content_type);
        response.raw_header_adjoin("Cross-Origin-Opener-Policy",  "same-origin");
        response.raw_header_adjoin("Cross-Origin-Embedder-Policy", "require-corp");
        response.streamed_body(data.as_ref());
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
    let rocket = rocket::build()
        .mount("/", routes![myfile]);

    Ok(rocket)
}
