use rocket::{State, response::content::Html};
use rocket::Route;
use std::path::PathBuf;
use crate::auth::jwt::JWT;
use crate::database::{SharedDatabase, UserRoll};
use crate::database::DBUser;
use std::fmt::Write as FmtWrite;
use log::{info, warn};

pub fn mount_admin() -> Vec<Route> {
    routes![
        get_admin_root,
        get_image_cache,
        cleanup_image_cache
    ]
}

#[derive(Serialize)]
struct Session {
    token: String,
    last_conn: std::time::SystemTime,
    user_id: String,
}

// TODO secure admin routes
/*
#[get("/admin/active_sessions")]
fn get_active_sessions() -> Json<Vec<Session>> {
    let mut res = Vec::new();

    for (token, data) in token_storage().inner().iter() {
        res.push(Session {
            token: String::from_utf8_lossy(token).to_string(),
            last_conn: data.0,
            user_id: (data.1).0.clone(),
        })
    }

    Json(res)
}
*/

#[get("/admin")]
fn get_admin_root(db: &State<SharedDatabase>, jwt: JWT) -> std::io::Result<Html<String>> {
    
    if jwt.user_roll != UserRoll::Admin {
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }

    let mut body = std::fs::read_to_string("./pages/admin.html")?;
    // generate user tr entries
    let users: Vec<DBUser> = db.get_all_users().map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;
    let user_html = users.iter().fold(String::with_capacity(users.len() * 32), |mut res, user| {
        writeln!(&mut res, r#"
            <tr>
                <td>{}</td>
                <td class="pw-change">
                    <input type="text"></input>
                    <button onclick="changePassword('{}')">Change</button>
                </td>
            </tr>"#, user.name, user.id.0).unwrap();
        res
    });

    body = body.replace("{{users}}", &user_html);

    Ok(Html(body))
}

use crate::fs::previews;

#[get("/admin/image_cache")]
fn get_image_cache() -> Option<Html<String>> {
    let mut total_size = 0;
    let mut total_count = 0;
    // [(count, size)]
    let mut sizes_from_res = Box::new([(0, 0); previews::ALLOWED_PREVIEW_RES.end as usize + 1]);

    for rfile in previews::cache_path().read_dir().ok()? {
        if let Ok(file) = rfile {
            let size = file.metadata().map(|md| md.len()).unwrap_or(0);
            total_size += size;
            total_count += 1;
            // get resolution of cached image
            let res: usize = file
                .file_name()
                .to_string_lossy()
                .split("_")
                .next()
                .and_then(|res_str| res_str.parse().ok())
                .unwrap_or(0);
            if res == 0 {
                warn!(
                    "Unknwon cached file resolution of \"{:?}\"",
                    file.file_name()
                );
            }
            sizes_from_res[res].0 += 1;
            sizes_from_res[res].1 += size;
        }
    }

    let page = format!(
        r#"
    <html>
        <body>
            <p>
                Total image bytes cached: {total_size} ({total_count} images)
            </p>
        </body>
    </html>
    "#,
        total_size = total_size,
        total_count = total_count
    );

    Some(Html(page))
}

/// max_size in mb
#[get("/admin/image_cache/cleanup?<max_size>")]
fn cleanup_image_cache(max_size: Option<u64>) -> Option<Html<String>> {
    if max_size.is_none() {
        warn!("max_size not specified, keeping max. 100mb");
    }
    let max_size = max_size.unwrap_or(100) * 1024 * 1024;
    info!(
        "Image cache cleanup, keeping newest {} mb",
        max_size / (1024 * 1024)
    );

    struct CacheFile {
        size: u64,
        // seconds
        age: u64,
        path: PathBuf,
    }

    let now = std::time::SystemTime::now();

    let mut all_files: Vec<CacheFile> = previews::cache_path()
        .read_dir()
        .ok()?
        .filter_map(|rfile| {
            rfile.ok().and_then(|file| {
                let md = file.metadata().ok()?;
                let modified = md.modified().ok()?;
                let age = now.duration_since(modified).ok()?.as_secs();
                Some(CacheFile {
                    size: md.len(),
                    age,
                    path: file.path(),
                })
            })
        })
        .collect();

    // will be most recent file first
    all_files.sort_by_key(|cf| cf.age);

    let mut retain_idx = 0;
    let mut retained_size = 0;

    while retain_idx < all_files.len() && retained_size + all_files[retain_idx].size <= max_size {
        retained_size += all_files[retain_idx].size;
        retain_idx += 1;
    }

    let (retain, delete) = all_files.split_at(retain_idx);
    let mut successful = true;
    for cf in delete {
        if let Err(e) = std::fs::remove_file(&cf.path) {
            warn!("Failed to delete cache image: {:?}", e);
            successful = false;
        }
    }

    let page = if successful {
        format!(
            r#"
        <html>
            <body>
                <h1>Cleanup successfull!</h1>
                <p>
                    Files deleted: {del_len}
                    Files remaining: {rem_len} ({rem_size}mb)
                </p>
            </body>
        </html>
        "#,
            del_len = delete.len(),
            rem_len = retain.len(),
            rem_size = retained_size / (1024 * 1024)
        )
    } else {
        r#"
        <html>
            <body>
                <h1>Cleanup failed. Have a look at the logs!</h1>
            </body>
        </html>
        "#
        .to_string()
    };

    Some(Html(page))
}
