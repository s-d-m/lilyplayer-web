#[macro_use]
extern crate rocket;

//use rocket::fs::NamedFile;
//use rocket::response::status::NotFound;
use std::path::PathBuf;

use std::io::Cursor;

use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;

use shuttle_service::ShuttleRocket;

//mod NamedFileWithHeaders;

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, world!"
}

// #[get("/<file..>")]
// async fn files(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
//     let path = Path::new("static/").join(file);
//     let mut response = NamedFile::open(&path).await.map_err(|e| NotFound(e.to_string()));
//     if let Ok(data) = response {
//         let mut res = data;
//         res.headers();
//         Ok(res)
//     } else {
//         response
//     }
// }



struct Person {
    name: String,
    age: u16,
    path: String
}

impl Person {
    fn from_id(filename: &str) -> Person {
        Person {
            name: String::from(filename),
            age: 18,
            path: String::from("Cargo.toml")
        }
    }
}



impl<'r> Responder<'r, 'static> for Person {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let string = format!("{}:{}", self.name, self.age);
        Response::build_from(string.respond_to(req)?)
            .raw_header("Cross-Origin-Opener-Policy",  "same-origin")
            .raw_header("Cross-Origin-Embedder-Policy", "require-corp")
            .header(ContentType::new("application", "x-person"))
            .ok()
    }
}

#[get("/person/<filename>")]
fn person(filename: &str) -> Option<Person> {
    Some(Person::from_id(filename))
}


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
        .mount("/", routes![hello, person, myfile]);

    Ok(rocket)
}
