#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use chrono::{prelude::*, DateTime, Local};
use diesel::{prelude::*, SqliteConnection};
use itertools::Itertools;
use rocket::http::RawStr;
use rocket_contrib::{serve::StaticFiles, templates::Template};
use std::collections::HashMap;

use pontjes::schema::gvb_stop_times;
use pontjes::schema::gvb_stops;
use pontjes::schema::pont_trips;

use pontjes::models;

#[database("pontjes_db")]
struct PontjesDb(SqliteConnection);

#[get("/")]
fn index(conn: PontjesDb) -> Template {
    match gvb_stops::table.load::<models::Stop>(&*conn) {
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
    let now = Local::now();
    let today = now.format("%Y%m%d").to_string();
    let tomorrow = (now + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();
    let time = now.format("%H:%M").to_string();

    println!("now      {}", now);
    println!("today    {}", today);
    println!("tomorrow {}", tomorrow);
    println!("time     {}", time);
    let trip_ids = gvb_stop_times::table
        .select(gvb_stop_times::dsl::trip_id)
        .filter(gvb_stop_times::dsl::stop_id.eq(sid.as_str()));
    let query = pont_trips::dsl::date
        .eq(today)
        .and(pont_trips::dsl::departure_time.gt(time))
        .or(pont_trips::dsl::date.eq(tomorrow))
        .and(pont_trips::dsl::trip_id.eq_any(trip_ids));
    let sql = diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query).to_string();
    println!("{:?}", sql);
    match pont_trips::table
        .filter(query)
        .order(pont_trips::dsl::date)
        .then_order_by(pont_trips::dsl::departure_time)
        .load::<models::Row>(&*conn)
    {
        Ok(results) => {
            let mut context: HashMap<String, Vec<models::Row>> = HashMap::new();
            for row in results {
                if !context.contains_key(&row.trip_id) {
                    context.insert(row.trip_id.clone(), Vec::new());
                }
                if let Some(d) = context.get_mut(&row.trip_id) {
                    d.push(row);
                }
            }
            println!("{:?}", &context);
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
