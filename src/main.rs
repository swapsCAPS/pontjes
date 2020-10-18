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

use pontjes::schema::calendar_dates as cd;
use pontjes::schema::gvb_stops;
use pontjes::schema::stop_times as st;
use pontjes::schema::stops;
use pontjes::schema::trips;

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

    let trips_that_have_sid: Vec<String> = st::table
        .inner_join(trips::table.on(trips::dsl::trip_id.eq(st::dsl::trip_id)))
        .inner_join(
            cd::table.on((cd::dsl::date.eq(&today).or(cd::dsl::date.eq(&tomorrow)))
                .and(cd::dsl::service_id.eq(trips::dsl::service_id))),
        )
        .filter(st::dsl::stop_id.eq(&sid))
        .select(st::dsl::trip_id)
        .distinct()
        .load(&*conn)
        .unwrap();

    let query = trips::table
        .inner_join(
            cd::table.on((cd::dsl::date.eq(&today).or(cd::dsl::date.eq(&tomorrow)))
                .and(cd::dsl::service_id.eq(trips::dsl::service_id))),
        )
        .inner_join(st::table.on(st::dsl::trip_id.eq(trips::dsl::trip_id)))
        .inner_join(stops::table.on(stops::dsl::stop_id.eq(st::dsl::stop_id)))
        .filter(
            trips::dsl::trip_id.eq_any(trips_that_have_sid).and(
                st::dsl::departure_time
                    .gt(time)
                    .or(cd::dsl::date.eq(&tomorrow)),
            ),
        )
        .order(cd::dsl::date)
        .then_order_by(st::dsl::departure_time)
        .select((
            cd::dsl::date,
            st::dsl::departure_time,
            stops::dsl::stop_name,
            st::dsl::stop_id,
            st::dsl::trip_id,
            st::dsl::stop_sequence,
        ));

    let sql = diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query);
    println!("query {:?}", sql);

    match query.load::<models::Row>(&*conn) {
        Ok(results) => {
            println!("results {:?}", results);
            let tuples: Vec<(String, models::Row)> = results
                .into_iter()
                .map(|r| (format!("{}{}", r.date, r.trip_id), r))
                .collect_vec();

            let group_map = tuples.into_iter().into_group_map();
            println!("group_map {:?}", group_map);

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
