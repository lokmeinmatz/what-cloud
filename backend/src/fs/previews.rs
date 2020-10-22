use crate::auth::hash_str_to_hex;
use std::path::Path;
use std::borrow::Borrow;
use crate::fs::to_abs_data_path;
use crate::fs::SharedDatabase;
use rocket::State;
use rocket::http::RawStr;
use crate::fs::NetFilePath;
use crate::fs::UserID;
use log::{info, warn, error};
use rocket::response::NamedFile;

#[derive(Responder)]
pub enum ImagePreviewResponse {
    #[response(status = 200)]
    Preview(NamedFile),
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 403)]
    WrongSize(&'static str),
    #[response(status = 406)]
    NoImage(&'static str),
    #[response(status = 404)]
    NotFound(()),
    // TODO allow even if cache doesn't work?
    #[response(status = 500)]
    ServerError(()),
}

fn is_deprecated_cache(src: &Path, cache: &Path) -> bool {
    let src_meta = std::fs::metadata(src);
    let cache_meta = std::fs::metadata(cache);

    match (src_meta, cache_meta) {
        (Ok(src_md), Ok(cache_md)) => {

            match (src_md.modified(), cache_md.modified()) {
                (Ok(src_mod), Ok(cache_mod)) => {
                    src_mod > cache_mod // if source is greater (newer) than cache file, recache
                }
                _ => true
            }

        },
        _ => true
    }
}

#[get("/preview/file?<path>&<token>&<resolution>", rank = 1)]
pub fn preview_image(path: NetFilePath, token: UserID, resolution: Option<u32>) -> ImagePreviewResponse {
    let abs_path = to_abs_data_path(&token, Borrow::<Path>::borrow(&path));
    if !abs_path.is_file() || abs_path.extension().map(|oss| oss != "png").unwrap_or(false) {
        return ImagePreviewResponse::NoImage("File needs to be an image (.png)");
    }

    let res = resolution.unwrap_or(256);
    if !(100..2048).contains(&res) {
        warn!("Tried to preview image with res = {}", res);
        return ImagePreviewResponse::WrongSize("Allowed res >= 100 & <= 2048")
    }

    let mut cache_dir = std::env::temp_dir();
    cache_dir.push("what-cloud");
    cache_dir.push("previews");

    if !cache_dir.exists() {
        if let Err(_) = std::fs::create_dir_all(&cache_dir) {
            warn!("Creating temp dir {:?} failed, can't cache files", &cache_dir);
            return ImagePreviewResponse::ServerError(());
        }
        info!("Created preview cache folder at {:?}", &cache_dir);

        let hashed_path = hash_str_to_hex("");

        let cached_file_name = format!("{}_{}.jpg", res, &hashed_path[0..10]);
        
        dbg!(&cached_file_name);

        cache_dir.push(cached_file_name);
        
        if is_deprecated_cache(&abs_path, &cache_dir) {
            // create new file
            if let Ok(src) = image::open(&abs_path) {
                let scaled = if res >= 500 {src.resize(res, res, image::imageops::FilterType::Nearest)} else {src.thumbnail(res, res)};
                if let Err(e) = scaled.save(&cache_dir) {
                    error!("Failed to save preview image: {:?}", e);
                    return ImagePreviewResponse::ServerError(());
                }
                info!("Cached new file {:?}", abs_path.file_name());
            } else {
                return ImagePreviewResponse::NoImage("couldn't open file, is it a image?");
            }
        }
    }
    match NamedFile::open(&cache_dir) {
        Ok(nf) => ImagePreviewResponse::Preview(nf),
        Err(e) => {
            error!("Failed to open cached file: {:?}", e);
            ImagePreviewResponse::ServerError(())
        }
    }

}

#[get("/preview/file?<path>&<shared_id>&<resolution>", rank = 2)]
pub fn preview_image_shared(
    mut path: NetFilePath,
    shared_id: &RawStr,
    db: State<SharedDatabase>,
    resolution: Option<u32>,
) -> ImagePreviewResponse {
    if let Some(se) = db.get_shared_entry(&shared_id) {
        
        path.add_prefix(&se.path);
        
        preview_image(path, se.user, resolution)
    } else {
        ImagePreviewResponse::NotFound(())
    }
}
