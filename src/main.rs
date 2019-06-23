#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rand;
extern crate serde;

mod paste_id;
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use rocket::http::ContentType;
use rocket::response::Content;
use rocket::Data;

use paste_id::PasteID;

use serde::{Deserialize, Serialize};

const ID_LENGTH: usize = 8;

#[post("/", data = "<data>")]
fn upload(data: Data, content_type: &ContentType) -> io::Result<String> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{id}\n", id = id);

    let mut content_type = format!("{}", content_type);

    if !content_type.starts_with("image") {
        content_type = "text/plain".to_string();
    }

    let mut metadata_file = File::create(format!("upload/{id}.metadata.json", id = id))?;
    let metadata = Metadata {
        content_type: content_type,
    };
    metadata_file.write_all(serde_json::to_string(&metadata).unwrap().as_bytes())?;

    data.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    content_type: String,
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Option<Content<File>> {
    let filename = format!("upload/{id}", id = id);

    let metadata_filename = format!("upload/{id}.metadata.json", id = id);

    let mut metadata_serialized = String::new();

    match File::open(&metadata_filename) {
        Err(_) => 0usize,
        Ok(mut file) => file.read_to_string(&mut metadata_serialized).unwrap(),
    };

    let metadata: Metadata = match serde_json::from_str(&metadata_serialized) {
        Err(_) => Metadata {
            content_type: "text/plain".to_string(),
        },
        Ok(md) => md,
    };

    let content_type = match ContentType::parse_flexible(&metadata.content_type) {
        Some(ct) => ct,
        None => ContentType::Plain,
    };

    File::open(&filename).map(|f| Content(content_type, f)).ok()
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
