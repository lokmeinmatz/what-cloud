#![feature(proc_macro_hygiene, decl_macro)]
#![feature(const_generics)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use log::{LevelFilter, info, error};
use simplelog::{Config, TerminalMode};
use std::path::PathBuf;
use rocket_cors::{Cors, AllowedOrigins};


mod api_mount;
mod auth;
mod database;
mod token_validizer;
mod fs;
mod config;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn cors() -> Cors {
    rocket_cors::CorsOptions::default().allowed_origins(AllowedOrigins::some_exact
        (&["http://localhost:8080", "http://localhost:8081"])).to_cors().unwrap()
}

fn main() {

    simplelog::TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).expect("simplelog failed");

    info!("Loading dotenv vars...");
    if let Err(e) = dotenv::dotenv() {
        error!("Dotenv error: {}", e);
    }

    config::init().expect("Failed to init config...");
    

    let db = database::SharedDatabase::new(config::db_path());



    rocket::ignite()
        .manage(db)
        .manage(token_validizer::ActiveTokenStorage::with_debug_access_token())
        .mount("/", routes![index])
        .mount("/api/", api_mount::mount_api())
        .attach(cors())
        .launch();
}
