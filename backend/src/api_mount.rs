use rocket::Route;

pub fn mount_api() -> Vec<Route> {
    routes![crate::auth::login, crate::fs::get_folder_content]
}