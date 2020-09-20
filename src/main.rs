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
        gvb_stops (stop_id) {
            stop_id -> Text,
            stop_code -> Text,
            stop_name -> Text,
            stop_lat -> Text,
            stop_lon -> Text,
            location_type -> Text,
            parent_station -> Text,
            stop_timezone -> Text,
            wheelchair_boarding -> Text,
            platform_code -> Text,
            zone_id -> Text,
        }
    }
    table! {
        gvb_stop_times (stop_id) {
            trip_id -> Text,
            stop_sequence -> Text,
            stop_id -> Text,
            stop_headsign -> Text,
            arrival_time -> Text,
            departure_time -> Text,
            pickup_type -> Text,
            drop_off_type -> Text,
            timepoint -> Text,
            shape_dist_traveled -> Text,
            fare_units_traveled -> Text,
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

allow_tables_to_appear_in_same_query!(pont_trips, gvb_stop_times); // sErioUsLy?!

use self::schema::gvb_stop_times;
use self::schema::gvb_stops;
use self::schema::pont_trips;

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Route {
    pub route_id: String,
    pub route_long_name: String,
}

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_code: String,
    pub stop_name: String,
    pub stop_lat: String,
    pub stop_lon: String,
    pub location_type: String,
    pub parent_station: String,
    pub stop_timezone: String,
    pub wheelchair_boarding: String,
    pub platform_code: String,
    pub zone_id: String,
}

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Row {
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
    match gvb_stops::table.load::<Stop>(&*conn) {
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
    let trip_ids = gvb_stop_times::table
        .select(gvb_stop_times::dsl::trip_id)
        .filter(gvb_stop_times::dsl::stop_id.eq(sid.as_str()));
    let query = pont_trips::dsl::date
        .eq(today)
        .and(pont_trips::dsl::departure_time.gt(time))
        .and(pont_trips::dsl::trip_id.eq_any(trip_ids));
    let sql = diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query).to_string();
    println!("{:?}", sql);
    match pont_trips::table
        .filter(query)
        .order(pont_trips::dsl::departure_time)
        .load::<Row>(&*conn)
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
