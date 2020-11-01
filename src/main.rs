#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use chrono::Utc;
use chrono_tz::Europe::Amsterdam;
use itertools::Itertools;
use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket_contrib::{databases::rusqlite, templates::Template};
use std::path::{Path, PathBuf};

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
struct PontjesDb(rusqlite::Connection);

#[get("/")]
fn index(conn: PontjesDb) -> Template {
    let mut stmt = conn
        .prepare(
            "
        select distinct s.stop_id, stop_name from routes as r
        inner join trips as t on t.route_id = r.route_id
        inner join stop_times as st on st.trip_id = t.trip_id
        inner join stops as s on s.stop_id = st.stop_id
        where agency_id = 'GVB' and r.route_url like '%veerboot%'
        order by stop_name;
        ",
        )
        .unwrap();

    let stops = stmt
        .query_map(&[], |row| models::Stop {
            stop_id: row.get(0),
            stop_name: row.get(1),
        })
        .unwrap()
        .map(|x| x.unwrap())
        .collect_vec();

    let context = models::IndexCtx {
        title: "Vanaf",
        stops,
    };

    Template::render("index", &context)
}

#[get("/upcoming-departures/<raw_sid>")]
fn upcoming_departures(conn: PontjesDb, raw_sid: &RawStr) -> Template {
    let now = Utc::now();
    let amsterdam_now = now.with_timezone(&Amsterdam);
    let today = amsterdam_now.format("%Y%m%d").to_string();
    let tomorrow = (amsterdam_now + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();
    let time = amsterdam_now.format("%H:%M").to_string();
    let sid = raw_sid.to_string();

    let mut stmt = conn
        .prepare(
            "
        select
          date,
          departure_time,
          stop_name,
          stop_sequence,
          s.stop_id,
          t.trip_id
        from trips as t
        inner join stop_times as st on st.trip_id=t.trip_id
        inner join stops as s on s.stop_id=st.stop_id
        inner join calendar_dates as cd on cd.service_id=t.service_id
        where
          (
            (date = :today and departure_time > :time) or date = :tomorrow
          ) and t.trip_id in (
              select distinct st.trip_id
              from stop_times as st
              where st.stop_id = :sid
            )
        order by date, departure_time;
        ",
        )
        .unwrap();

    let results = stmt
        .query_map_named(
            &[
                (":today", &today),
                (":tomorrow", &tomorrow),
                (":sid", &sid),
                (":time", &time),
            ],
            |row| models::Row {
                date: row.get(0),
                departure_time: row.get(1),
                stop_name: row.get(2),
                stop_sequence: row.get(3),
                stop_id: row.get(4),
                trip_id: row.get(5),
            },
        )
        .unwrap()
        .map(|x| x.unwrap())
        .collect_vec();

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

    list_items.truncate(64);

    let stop_name: String = conn
        .query_row(
            "select stop_name from stops where stop_id = ?;",
            &[&sid],
            |row| row.get(0),
        )
        .unwrap();

    let context = models::DeparturesCtx {
        title: &format!("Van {}", stop_name),
        list_items,
    };

    Template::render("upcoming-departures", &context)
}

fn main() {
    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes![index, upcoming_departures])
        .mount("/public", routes![cached_files])
        .attach(Template::fairing())
        .launch();
}
