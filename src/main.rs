#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use chrono::Utc;
use chrono_tz::Europe::Amsterdam;
use diesel::{prelude::*, SqliteConnection};
use itertools::Itertools;
use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket_contrib::templates::Template;
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
            let context = models::IndexCtx {
                stops: results,
                title: "Vanaf",
            };
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

    // Get all trip ids for this stop_id
    let trip_ids = gvb_stop_times::table
        .select(gvb_stop_times::dsl::trip_id)
        .filter(gvb_stop_times::dsl::stop_id.eq(sid));
    // Get all enriched rows with these trip_ids and current time
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

            let mut list_items: Vec<models::ListItem> = group_map
                .values()
                // TODO The length filter is prolly too naive
                .filter(|row| row.len() > 1 && row[row.len() - 1].stop_id != sid)
                .map(|trip| {
                    let active_stop = trip.iter().find(|x| x.stop_id == sid).unwrap();
                    let last = &trip[trip.len() - 1];
                    let mut rest_stops = trip
                        .iter()
                        .filter(|x| x.stop_id != sid)
                        .map(|row| models::ListItemStop {
                            date: &row.date,
                            time: &row.departure_time,
                            stop_name: &row.stop_name,
                        })
                        .collect_vec();

                    rest_stops.pop();

                    models::ListItem {
                        date: &active_stop.date,
                        time: &active_stop.departure_time,
                        rest_stops,
                        end_stop: models::ListItemStop {
                            date: &last.date,
                            time: &last.departure_time,
                            stop_name: &last.stop_name,
                        },
                    }
                })
                .sorted_by_key(|list_item| (list_item.date, list_item.time))
                .collect_vec();

            list_items.truncate(32);

            match gvb_stops::table.find(sid).first::<models::Stop>(&*conn) {
                Ok(stop) => {
                    let context = models::DeparturesCtx {
                        title: &format!("Van {}", stop.stop_name),
                        requested_stop: sid,
                        list_items,
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
