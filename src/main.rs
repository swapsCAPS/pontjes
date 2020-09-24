#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use chrono::Local;
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
    match gvb_stops::table
        .order(gvb_stops::dsl::stop_name)
        .load::<models::Stop>(&*conn)
    {
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

    let trip_ids = gvb_stop_times::table
        .select(gvb_stop_times::dsl::trip_id)
        .filter(gvb_stop_times::dsl::stop_id.eq(sid.as_str()));
    let query = pont_trips::dsl::date
        .eq(today)
        .and(pont_trips::dsl::departure_time.gt(time))
        .or(pont_trips::dsl::date.eq(tomorrow))
        .and(pont_trips::dsl::trip_id.eq_any(trip_ids));

    match pont_trips::table
        .filter(query)
        .order(pont_trips::dsl::date)
        .then_order_by(pont_trips::dsl::departure_time)
        .load::<models::Row>(&*conn)
    {
        Ok(results) => {
            let tuples: Vec<(String, models::Row)> = results
                .into_iter()
                .map(|r| (format!("{}{}", r.date, r.trip_id), r))
                .collect_vec();

            let group_map = tuples.into_iter().into_group_map();

            let mut data: Vec<&Vec<models::Row>> = group_map
                .values()
                .filter(|row| row[row.len() - 1].stop_id != sid.as_str())
                .sorted_by(|a, b| {
                    let a_stop_id = pontjes::get_requested_stop(a, sid.as_str());
                    let b_stop_id = pontjes::get_requested_stop(b, sid.as_str());

                    Ord::cmp(&a_stop_id, &b_stop_id)
                })
                .collect();

            data.truncate(10);

            Template::render("upcoming-departures", &data)
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
