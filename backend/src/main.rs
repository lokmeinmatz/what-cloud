#![feature(proc_macro_hygiene, decl_macro, const_generics)]
#[macro_use] extern crate rocket;
//#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use log::{LevelFilter, info, error};
use simplelog::{Config, TerminalMode};
use rocket_cors::{Cors, AllowedOrigins};


mod api_mount;
mod auth;
mod database;
mod token_validizer;
mod fs;
mod config;
mod icons;
mod admin;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn cors() -> Cors {
    rocket_cors::CorsOptions::default().allowed_origins(AllowedOrigins::some_exact
        (&["http://localhost:8080", "http://192.168.178.38:8080"])).to_cors().unwrap()
}


#[launch]
fn rocket() -> rocket::Rocket {

    simplelog::TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).expect("simplelog failed");

    info!("Loading dotenv vars...");
    if let Err(e) = dotenv::dotenv() {
        error!("Dotenv error: {}", e);
    }

    config::init().expect("Failed to init config...");
    token_validizer::init(false);

    let db = database::SharedDatabase::new(config::db_path());

    let mut api_routes = api_mount::mount_api();
    api_routes.extend_from_slice(&admin::mount_admin());

    rocket::ignite()
        .manage(db)
        .manage(icons::IconsCache::empty())
        .manage(token_validizer::ActiveTokenStorage::with_debug_access_token())
        .mount("/", routes![index])
        .mount("/api/", api_routes)
        .attach(cors())
}
