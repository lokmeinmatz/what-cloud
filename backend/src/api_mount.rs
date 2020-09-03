use rocket::Route;

pub fn mount_api() -> Vec<Route> {
    routes![
        crate::auth::login, 
        crate::auth::logout, 
        crate::fs::get_folder_content, 
        crate::fs::metadata::get_metadata,
        crate::fs::download_file,
        crate::icons::icons_get,
        crate::auth::my_user,
        crate::auth::my_user_not_loggedin,
    ]
}