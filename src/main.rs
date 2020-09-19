#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

use diesel::{prelude::*, result::QueryResult, SqliteConnection};
use rocket::http::RawStr;
use rocket_contrib::{serve::StaticFiles, templates::Template};
use std::collections::HashMap;

mod schema {
    table! {
        pont_stops (stop_id) {
            stop_id -> Text,
            stop_name -> Text,
        }
    }
}

use self::schema::pont_stops::dsl::*;

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
}

#[database("pontjes_db")]
struct PontjesDb(SqliteConnection);

#[get("/")]
fn index(conn: PontjesDb) -> Template {
    match pont_stops.load::<Stop>(&*conn) {
        Ok(results) => {
            let mut context = HashMap::new();
            context.insert("stops", results);
            Template::render("index", &context)
        }
        Err(e) => {
            println!("Error! {}", e);
            Template::render("error", ())
        }
    }
}

#[get("/departures/<id>")]
fn stop(conn: PontjesDb, id: &RawStr) {
    println!("stop_id {}", id.as_str());
}

fn main() {
    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes![index, stop])
        .mount("/public", StaticFiles::from("./public"))
        .attach(Template::fairing())
        .launch();
}
