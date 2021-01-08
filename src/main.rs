//use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{fs::File, time::SystemTime};

use log::{debug, info, warn};

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

static TMP_DIR: &str = "./tmp";

fn directory_cleanup() -> Result<()> {
    for dir in std::fs::read_dir(TMP_DIR)? {
        let dir = dir?;
        let metadata = dir.metadata()?;
        if let Ok(time) = metadata.created() {
            let now = SystemTime::now();
            let diff = now.duration_since(time).unwrap();
            if diff.as_secs() > 600 {
                let dir_path = dir.path();
                debug!("Deleting directory {:?}", dir_path);
                match std::fs::remove_dir_all(&dir_path) {
                    Ok(_) => {
                        debug!("Removed directory {:?}", dir_path);
                    }
                    Err(_) => {
                        warn!("Cannot remove directory {:?}", dir_path);
                    }
                }
            }
        } else {
            warn!("Not supported on this platform or filesystem");
        }
    }
    Ok(())
}

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

    // Remove old temporary files
    directory_cleanup()?;

    // Create unique session id and directory
    let session_id = Uuid::new_v4();
    let session_dir = format!("{}/{}", TMP_DIR, session_id);
    std::fs::create_dir(&session_dir)?;

    // Load the first file
    let mut field = payload.try_next().await.unwrap().unwrap();
    let content_type = field.content_disposition().unwrap();
    let filename = content_type.get_filename().unwrap();
    let filepath = format!("{}/{}", session_dir, sanitize_filename::sanitize(filename));
    let final_filename = format!("{}.cc", &filename);
    let final_filepath = format!("{}.cc", &filepath);

    // Create file
    let mut f = File::create(&filepath)?;

    // Write bytes to file
    while let Some(chunk) = field.next().await {
        let data = chunk?;
        f = web::block(move || f.write_all(&data).map(|_| f)).await?;
    }

    // Dump to hex and put in c array with xxd
    let status = Command::new("xxd")
        .current_dir(session_dir)
        .arg("-i")
        .arg(filename)
        .arg(&final_filename)
        .status()
        .expect("Failed");
    info!("Conversion exited with: {}", status);

    // Remove pre converted file
    std::fs::remove_file(filepath)?;

    // Setup content disposition with correct filename
    let cd = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(String::from(final_filename))],
    };

    // Return converted file
    Ok(NamedFile::open(final_filepath)?.set_content_disposition(cd))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create temporary directory for file conversions
    std::fs::create_dir_all(TMP_DIR)?;

    // Setup logging
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // Setup http server
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
