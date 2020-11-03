use crate::auth::hash_str_to_hex;
use crate::fs::to_abs_data_path;
use crate::fs::NetFilePath;
use crate::fs::SharedDatabase;
use crate::fs::UserID;
use log::{error, info, warn};
use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket::State;
use std::borrow::Borrow;
use std::ops::Range;
use std::path::Path;

#[derive(Responder)]
pub enum ImagePreviewResponse {
    #[response(status = 200)]
    Preview(NamedFile),
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

pub const ALLOWED_PREVIEW_RES: Range<u32> = 100..2048;

fn is_deprecated_cache(src: &Path, cache: &Path) -> bool {
    let src_meta = std::fs::metadata(src);
    let cache_meta = std::fs::metadata(cache);

    match (src_meta, cache_meta) {
        (Ok(src_md), Ok(cache_md)) => {
            match (src_md.modified(), cache_md.modified()) {
                (Ok(src_mod), Ok(cache_mod)) => {
                    src_mod > cache_mod // if source is greater (newer) than cache file, recache
                }
                _ => true,
            }
        }
        _ => true,
    }
}

const ALLOWED_FORMATS: [&'static str; 3] = ["png", "jpeg", "jpg"];

pub fn cache_path() -> std::path::PathBuf {
    let mut cache_dir = std::env::temp_dir();
    cache_dir.push("what-cloud");
    cache_dir.push("previews");
    cache_dir
}

#[get("/preview/file?<path>&<token>&<resolution>", rank = 1)]
pub async fn preview_image(
    path: NetFilePath,
    token: UserID,
    resolution: Option<u32>,
) -> ImagePreviewResponse {
    let abs_path = to_abs_data_path(&token, Borrow::<Path>::borrow(&path));
    if !abs_path.is_file() {
        return ImagePreviewResponse::NotFound(());
    }

    // fails if extension is unknwon or no extension present
    if !abs_path
        .extension()
        .map(|oss| ALLOWED_FORMATS.iter().any(|fmt| fmt == &oss))
        .unwrap_or(false)
    {
        return ImagePreviewResponse::NoImage("File needs to be an image (.png)");
    }

    let res = resolution.unwrap_or(256);
    if !ALLOWED_PREVIEW_RES.contains(&res) {
        warn!("Tried to preview image with res = {}", res);
        return ImagePreviewResponse::WrongSize("Allowed res >= 100 & <= 2048");
    }

    let mut cache_dir = cache_path();

    if !cache_dir.exists() {
        if let Err(_) = std::fs::create_dir_all(&cache_dir) {
            warn!(
                "Creating temp dir {:?} failed, can't cache files",
                &cache_dir
            );
            return ImagePreviewResponse::ServerError(());
        }
        info!("Created preview cache folder at {:?}", &cache_dir);
    }

    let hashed_path = hash_str_to_hex(abs_path.to_str().unwrap());

    let cached_file_name = format!("{}_{}.jpg", res, &hashed_path[0..30]);

    cache_dir.push(cached_file_name);

    if is_deprecated_cache(&abs_path, &cache_dir) {
        // create new file
        let open_start = std::time::Instant::now();
        if let Ok(src) = image::open(&abs_path) {
            dbg!(open_start.elapsed());
            let resize_start = std::time::Instant::now();
            let scaled = if res >= 500 {
                src.resize(res, res, image::imageops::FilterType::Nearest)
            } else {
                src.thumbnail(res, res)
            };
            dbg!(resize_start.elapsed());
            if let Err(e) = scaled.save(&cache_dir) {
                error!("Failed to save preview image: {:?}", e);
                return ImagePreviewResponse::ServerError(());
            }
            info!("Cached new file {:?}", cache_dir.file_name());
        } else {
            return ImagePreviewResponse::NoImage("couldn't open file, is it a image?");
        }
    }

    match NamedFile::open(&cache_dir).await {
        Ok(nf) => ImagePreviewResponse::Preview(nf),
        Err(e) => {
            error!("Failed to open cached file {:?}: {:?}", cache_dir, e);
            ImagePreviewResponse::ServerError(())
        }
    }
}

#[get("/preview/file?<path>&<shared_id>&<resolution>", rank = 2)]
pub async fn preview_image_shared(
    mut path: NetFilePath,
    shared_id: &RawStr,
    db: State<'_, SharedDatabase>,
    resolution: Option<u32>,
) -> ImagePreviewResponse {
    if let Some(se) = db.get_shared_entry(&shared_id) {
        path.add_prefix(&se.path);

        preview_image(path, se.user, resolution).await
    } else {
        ImagePreviewResponse::NotFound(())
    }
}
