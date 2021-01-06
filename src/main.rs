extern crate actix_web;
extern crate uuid;
extern crate log;
extern crate env_logger;

//use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use log::{info};

use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{
    http::header::{self, DispositionParam, DispositionType},
    middleware, web, App, HttpRequest, HttpServer, Result,
};
use fs::NamedFile;
use futures::{StreamExt, TryStreamExt};
use header::ContentDisposition;

use uuid::Uuid;

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse()?;
    Ok(NamedFile::open(path)?)
}

async fn xxd(mut payload: Multipart) -> Result<NamedFile> {
    // Maximum filesize in bytes
    // let mut max_filesize = 20971520;
    // if !env::var("MAX_FILESIZE").is_err() {
    //     max_filesize = env::var("MAX_FILESIZE").unwrap().parse::<u64>().unwrap();
    // }

    let session_id = Uuid::new_v4();
    let session_dir = format!("./tmp/{}", session_id);
    std::fs::create_dir(session_dir.clone())?;

    let mut field = payload.try_next().await.unwrap().unwrap();
    let content_type = field.content_disposition().unwrap();
    let filename = content_type.get_filename().unwrap();
    let filepath = format!("{}/{}", session_dir, sanitize_filename::sanitize(filename));
    let final_filepath = format!("{}.cc", filepath.clone());

    let mut f = File::create(filepath.clone())?;

    while let Some(chunk) = field.next().await {
        let data = chunk?;
        f = web::block(move || f.write_all(&data).map(|_| f)).await?;
    }

    // Dump to hex and put in c array with xxd
    let status = Command::new("xxd")
        .arg("-i")
        .arg(filepath.clone())
        .arg(final_filepath.clone())
        .status()
        .expect("Failed");
    info!("Conversion exited with: {}", status);

    std::fs::remove_file(filepath)?;
    
    let final_filename = format!("{}.cc", filename);

    let cd = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(String::from(final_filename))],
    };

    Ok(NamedFile::open(final_filepath)?.set_content_disposition(cd))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./tmp")?;

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .route("/xxd", web::post().to(xxd))
            .service(fs::Files::new("/", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
