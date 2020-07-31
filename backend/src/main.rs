#![feature(proc_macro_hygiene, decl_macro)]
#![feature(const_generics)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use log::{LevelFilter, info};
use simplelog::{Config, TerminalMode};
use structopt::StructOpt;
use std::path::PathBuf;
use rocket_cors::{Cors, AllowedOrigins};


mod api_mount;
mod auth;
mod database;
mod token_validizer;
mod fs;
mod config;

#[derive(StructOpt)]
#[structopt(name = "what-cloud")]
struct CMDArgs {
    #[structopt(short, long, parse(from_os_str))]
    database_path: Option<PathBuf>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn cors() -> Cors {
    rocket_cors::CorsOptions::default().allowed_origins(AllowedOrigins::some_exact
        (&["http://localhost:8080", "http://localhost:8081"])).to_cors().unwrap()
}

fn main() {
    let args: CMDArgs = CMDArgs::from_args();
    simplelog::TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed);
    let db = database::SharedDatabase::new(args.database_path.unwrap_or_else(|| PathBuf::from
        ("database.sqlite")).as_path());

    config::init().expect("Failed to init config...");
    info!("config initialized");


    rocket::ignite()
        .manage(db)
        .manage(token_validizer::ActiveTokenStorage::empty())
        .mount("/", routes![index])
        .mount("/api/", api_mount::mount_api())
        .attach(cors())
        .launch();
}
