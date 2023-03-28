#[macro_use]
extern crate rocket;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use chrono_tz::Europe::Amsterdam;
use chrono::{Utc, DateTime, NaiveDate, NaiveDateTime, TimeZone};
use rocket::fs::NamedFile;
use rocket::response::Redirect;
use rocket_dyn_templates::Template;
use std::fs;
use std::path::{Path, PathBuf};

pub mod controllers;
pub mod models;
pub mod queries;
pub mod utils;

use utils::PontjesDb;

lazy_static! {
    static ref DOWNLOAD_DATE: Option<String> = fs::read_to_string("/data/download_date").ok();
}

#[get("/")]
async fn index(db: PontjesDb) -> Template {
    match controllers::index(db).await {
        Ok(context) => Template::render("index", context),
        Err(e) => {
            error!("/index Encountered unexpected error: {}", e);
            Template::render(
                "error",
                models::MainCtx {
                    title: String::from("Oeps... Stuk"),
                    page_title: String::from("pont.app"),
                    page_description: String::from(""),
                    feed_info: None,
                    download_date: None,
                    content: None,
                },
            )
        }
    }
}

// TODO WIP
#[get("/upcoming-departures/<raw_sid>?<dt>")]
async fn upcoming_departures(db: PontjesDb, raw_sid: &str, dt: Option<&str>) -> Result<Template, Redirect> {
    // If we can't parse correctly we want to log an error and return a NaiveDateTime for the
    // current datetime
    let naive_date = match dt {
        Some(date_time_str) => {
            NaiveDateTime::parse_from_str(date_time_str, "%Y-%m-%dT%H:%M").unwrap_or_else(|_| {
                warn!("Could not parse string {} to NaiveDateTimei, going for default of Utc::now()", date_time_str);
                NaiveDateTime::from_timestamp_opt(Utc::now().timestamp_millis() / 1000, 0).expect("What?! This really should not happen")
            })
        },
        None => {
            NaiveDateTime::from_timestamp_opt(Utc::now().timestamp_millis() / 1000, 0).expect("What?! This really should not happen")
        }
    };

    let amsterdam_date_time = Amsterdam.from_local_datetime(&naive_date).unwrap();

    debug!("amsterdam_date_time {}", amsterdam_date_time);

    match controllers::upcoming_departures(db, raw_sid.to_string(), amsterdam_date_time).await {
        Ok(context) => Ok(Template::render("upcoming-departures", context)),
        Err(e) => {
            warn!(
                "/upcoming-departures/{} encountered unexpected error: {}",
                raw_sid, e
            );
            Err(Redirect::to("/"))
        }
    }
}

#[get("/public/<file..>")]
async fn public(file: PathBuf) -> Option<models::CachedFile> {
    NamedFile::open(Path::new("public/").join(file))
        .await
        .ok()
        .map(|nf| models::CachedFile(nf))
}

// NOTE Service_worker needs to be hosted from root
// NOTE Not hard coding the path, otherwise recompile is needed when changing sw file name
//      This does mean that everything in ./public/scripts is hosted at `/`, but we don't care.
#[get("/<sw>")]
async fn service_worker(sw: &str) -> Option<models::CachedFile> {
    NamedFile::open(Path::new("public").join("scripts").join(sw))
        .await
        .ok()
        .map(|nf| models::CachedFile(nf))
}

#[launch]
fn rocket() -> _ {
    pretty_env_logger::init();

    rocket::build()
        .attach(PontjesDb::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![index, upcoming_departures, public, service_worker],
        )
}
