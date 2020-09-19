#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

use chrono::{prelude::*, DateTime, Local};
use diesel::{prelude::*, result::QueryResult, SqliteConnection};
use rocket::http::RawStr;
use rocket_contrib::{serve::StaticFiles, templates::Template};
use std::collections::HashMap;

mod schema {
    table! {
        pont_routes (route_id) {
            route_id -> Text,
            route_long_name -> Text,
        }
    }
    table! {
        pont_stops (stop_id) {
            stop_id -> Text,
            stop_name -> Text,
        }
    }
    table! {
        pont_trips (trip_id) {
          route_long_name -> Text,
          date -> Text,
          departure_time -> Text,
          stop_name -> Text,
          stop_id -> Text,
          trip_headsign -> Text,
          trip_id -> Text,
          stop_sequence -> Text,
        }
    }
}

use self::schema::pont_routes::dsl::*;
use self::schema::pont_stops::dsl::*;
use self::schema::pont_trips::dsl::{stop_id as pt_stop_id, *};

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Route {
    pub route_id: String,
    pub route_long_name: String,
}

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
}

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Trip {
    pub route_long_name: String,
    pub date: String,
    pub departure_time: String,
    pub stop_name: String,
    pub stop_id: String,
    pub trip_headsign: String,
    pub trip_id: String,
    pub stop_sequence: String,
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

#[get("/upcoming-departures/<sid>")]
fn stop(conn: PontjesDb, sid: &RawStr) -> Template {
    let now: DateTime<Local> = Local::now();
    println!("now {}", now);
    let today = now.format("%Y%m%d").to_string();
    println!("today {}", today);
    let time = now.format("%H:%M").to_string();
    println!("time {}", time);
    let query = date
        .eq(today)
        .and(pt_stop_id.eq(sid.as_str()))
        .and(departure_time.gt(time))
        .and(stop_sequence.eq("1"));
    match pont_trips
        .filter(query)
        .order(departure_time)
        .load::<Trip>(&*conn)
    {
        Ok(results) => {
            println!("results {:?}", results);
            let mut context = HashMap::new();
            context.insert("trips", results);
            Template::render("upcoming-departures", &context)
        }
        Err(e) => {
            println!("Error! {}", e);
            Template::render("error", ())
        }
    }
}

fn main() {
    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes![index, stop])
        .mount("/public", StaticFiles::from("./public"))
        .attach(Template::fairing())
        .launch();
}
