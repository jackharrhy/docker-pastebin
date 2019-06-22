#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rand;

mod paste_id;
#[cfg(test)] mod tests;

use std::io;
use std::fs::File;
use std::path::Path;

use rocket::Outcome;
use rocket::Data;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use rocket::response::content;

use paste_id::PasteID;

const ID_LENGTH: usize = 8;

struct ContentType(String);

#[derive(Debug)]
enum ContentTypeError {
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ContentType {
    type Error = ContentTypeError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("ContentType").collect();
        match keys.len() {
            0 => Outcome::Success(ContentType("text/plain".to_string())),
            1 => Outcome::Success(ContentType(keys[0].to_string())),
            _ => Outcome::Failure((Status::BadRequest, ContentTypeError::Invalid)),
        }
    }
}

#[post("/", data = "<data>")]
fn upload(data: Data, content_type: ContentType) -> io::Result<String> {
    println!("Got ContentType of: {}", content_type.0);
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{id}\n", id = id);

    data.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Option<content::Plain<File>> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).map(|f| content::Plain(f)).ok()
}

#[get("/")]
fn index() -> &'static str {
    "pastebin

  USAGE

    POST /

      accepts raw data in the body of the request and responds with a URL of
      a page containing the body's content

      EXAMPLE: curl --data-binary @file.txt http://example.com/pastebin

    GET /<id>

      retrieves the content for the paste with id `<id>`"
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, upload, retrieve])
}

fn main() {
    rocket().launch();
}
