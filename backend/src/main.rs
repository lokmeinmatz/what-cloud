#![feature(proc_macro_hygiene, decl_macro, min_const_generics)]
#[macro_use]
extern crate rocket;
//#[macro_use] extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use log::{error, info, LevelFilter};
use simplelog::{Config, TerminalMode};
// use rocket_cors::{Cors, AllowedOrigins};

mod admin;
mod api_mount;
mod auth;
mod config;
mod database;
mod fs;
mod icons;
mod utils;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

/*
fn cors() -> Cors {
    rocket_cors::CorsOptions::default().allowed_origins(AllowedOrigins::some_exact
        (&["http://localhost:8080", "http://192.168.178.38:8080", "https:cloud.matthiaskind.com"])).to_cors().unwrap()
}
*/

#[launch]
fn rocket() -> rocket::Rocket {
    simplelog::TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
        .expect("simplelog failed");

    info!("Loading dotenv vars...");
    if let Err(e) = dotenv::dotenv() {
        error!("Dotenv error: {}", e);
    }

    config::init().expect("Failed to init config...");

    if !config::data_path().is_dir() {
        error!("The data_path doesn't exist or isn't a dir");
        panic!("Create folder first.");
    }

    let db = database::SharedDatabase::new(config::db_path());

    info!("Cache path: {:?}", crate::fs::previews::cache_path());

    let mut api_routes = api_mount::mount_api();
    api_routes.extend_from_slice(&admin::mount_admin());

    rocket::ignite()
        .manage(db)
        .manage(icons::IconsCache::empty())
        .mount("/", routes![index])
        .mount("/api/", api_routes)
    //    .attach(cors())
}
