#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use chrono::{Local, TimeZone, Utc};
use chrono_tz::Europe::Amsterdam;
use diesel::{prelude::*, SqliteConnection};
use itertools::Itertools;
use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pontjes::schema::gvb_stop_times;
use pontjes::schema::gvb_stops;
use pontjes::schema::pont_trips;

use pontjes::models;

struct CachedFile(NamedFile);
impl<'r> rocket::response::Responder<'r> for CachedFile {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'r> {
        rocket::Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", "max-age=86400")
            .ok()
    }
}

#[derive(Serialize)]
struct Context<'a> {
    requested_stop: &'a str,
    departures: Vec<&'a Vec<models::Row>>,
}

#[get("/<file..>")]
fn cached_files(file: PathBuf) -> Option<CachedFile> {
    NamedFile::open(Path::new("public/").join(file))
        .ok()
        .map(|nf| CachedFile(nf))
}

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
            let mut context = HashMap::new();
            context.insert("msg", String::from("oops"));
            Template::render("error", context)
        }
    }
}

#[get("/upcoming-departures/<sid>")]
fn stop(conn: PontjesDb, sid: &RawStr) -> Template {
    let now = Utc::now();
    let amsterdam_now = now.with_timezone(&Amsterdam);
    let today = amsterdam_now.format("%Y%m%d").to_string();
    let tomorrow = (amsterdam_now + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();
    let time = amsterdam_now.format("%H:%M").to_string();
    let sid = sid.as_str();

    let trip_ids = gvb_stop_times::table
        .select(gvb_stop_times::dsl::trip_id)
        .filter(gvb_stop_times::dsl::stop_id.eq(sid));
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
                .filter(|row| row[row.len() - 1].stop_id != sid)
                .sorted_by(|a, b| {
                    let a_stop_id = pontjes::get_requested_stop(a, sid);
                    let b_stop_id = pontjes::get_requested_stop(b, sid);

                    Ord::cmp(&a_stop_id, &b_stop_id)
                })
                .collect();

            data.truncate(32);

            let context = Context {
                requested_stop: sid,
                departures: data,
            };
            Template::render("upcoming-departures", &context)
        }
        Err(e) => {
            println!("Error! {}", e);
            let mut context = HashMap::new();
            context.insert("msg", String::from("oops"));
            Template::render("error", context)
        }
    }
}

fn main() {
    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes![index, stop])
        .mount("/public", routes![cached_files])
        .attach(Template::fairing())
        .launch();
}
