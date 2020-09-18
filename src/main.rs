#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;

use rocket::http::{
    RawStr,
};

use diesel::{SqliteConnection, result::QueryResult, prelude::*};

use rocket_contrib::{
    templates::Template,
    serve::StaticFiles,
};

use std::collections::HashMap;

mod schema {
    table! {
        pont_stops (stop_id) {
            stop_name -> Text,
            stop_id -> Text,

        }
    }
}

use self::schema::pont_stops;
use self::schema::pont_stops::dsl::*;

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
}

#[database("pontjes_db")]
struct PontjesDb(SqliteConnection);

#[get("/")]
fn index(conn: PontjesDb) {

    let results = pont_stops.load::<Stop>(&*conn);
    println!("results {:?}", results);
}

fn main() {
    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes![index])
        .attach(Template::fairing())
        .launch();
}


