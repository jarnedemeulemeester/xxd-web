#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use rocket::Data;
use rocket::http::{ContentType, Status};
use rocket::http::hyper::header::{ ContentDisposition, DispositionType, DispositionParam, Charset};
use rocket::response::status::NotFound;
use rocket::response::Response;
use rocket_contrib::serve::StaticFiles;

use rocket_multipart_form_data::{MultipartFormDataOptions, MultipartFormData, MultipartFormDataField, FileField};

#[post("/xxd", data = "<data>")]
fn xxd(content_type: &ContentType, data: Data) -> Result<Response, NotFound<String>> {

    // Multipart Form setup
    let mut options = MultipartFormDataOptions::new();
    options.allowed_fields.push(MultipartFormDataField::file("file").size_limit(20971520));
    let multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let file = multipart_form_data.files.get("file");

    // let mut path = PathBuf::new();
    let mut new_path = PathBuf::new();
    let mut filename: &str = "";

    if let Some(file) = file {
        println!("File received");
        match file {
            FileField::Single(file) => {
                let _content_type = &file.content_type;
                let _file_name = &file.file_name;
                let _path = &file.path;
                new_path = _path.to_path_buf();
                new_path.set_extension("cc");
                filename = &_file_name.as_ref().unwrap();
                println!("{:?}", _path);
                
                // Covert to c array
                let status = Command::new("xxd")
                    .arg("-i")
                    .arg(_path.to_path_buf())
                    .arg(&new_path)
                    .status()
                    .expect("Failed");
                println!("Conversion exited with: {}", status);
            }
            FileField::Multiple(_bytes) => {
                
            }
        }
    }
    // Remove file extention and change to .cc
    // let filename_parts: Vec<&str> = filename.split(".").collect();
    // let filename_without_ext = &filename_parts[0..filename_parts.len()-1].join(".");
    // let final_filename = [filename_without_ext, ".", "cc"].concat();

    let final_filename = [filename, ".cc"].concat();

    let response = Response::build()
        .status(Status::Ok)
        .header(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(
              Charset::Iso_8859_1, // The character set for the bytes of the filename
              None, // The optional language tag (see `language-tag` crate)
              (&final_filename).as_bytes().to_vec() // the actual bytes of the filename
            )]
        })
        .sized_body(File::open(new_path).map_err(|e| NotFound(e.to_string()))?)
        .ok();

    response
}

fn main() {
    rocket::ignite()
        .mount("/", routes![xxd])
        .mount("/", StaticFiles::from("./static"))
        .launch();
}