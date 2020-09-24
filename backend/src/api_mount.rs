use rocket::Route;

pub fn mount_api() -> Vec<Route> {
    routes![
        crate::auth::login, 
        crate::auth::logout, 
        crate::fs::get_node_data_shared,
        crate::fs::get_node_data,
        crate::fs::download_file,
        crate::fs::shared::update_folder_share,
        crate::fs::shared::get_my_shared,
        crate::icons::icons_get,
        crate::auth::my_user,
        crate::auth::my_user_not_loggedin,
    ]
}