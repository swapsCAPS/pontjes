#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use chrono::Utc;
use chrono_tz::Europe::Amsterdam;
use itertools::Itertools;
use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket_contrib::templates::Template;
use std::fs;
use std::path::{Path, PathBuf};

use pontjes::{get_feed_info, models, parse_gtfs_time, PontjesDb};

struct CachedFile(NamedFile);

impl<'r> rocket::response::Responder<'r> for CachedFile {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'r> {
        rocket::Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", "max-age=86400")
            .ok()
    }
}

lazy_static! {
    static ref DOWNLOAD_DATE: Option<String> = fs::read_to_string("/data/download_date").ok();
}

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

    let feed_info = get_feed_info(&conn);

    let context = models::MainCtx {
        title: String::from("Vanaf"),
        feed_info,
        download_date: fs::read_to_string("/data/download_date").ok(),
        content: models::Content::IndexCtx { stops },
    };

    Template::render("index", &context)
}

#[get("/upcoming-departures/<raw_sid>")]
fn upcoming_departures(conn: PontjesDb, raw_sid: &RawStr) -> Template {
    let now = Utc::now();
    debug!("now {}", now);
    let amsterdam_now = now.with_timezone(&Amsterdam);
    debug!("amsterdam_now {}", amsterdam_now);
    let today = amsterdam_now.format("%Y%m%d").to_string();
    debug!("today {}", today);
    let tomorrow = (amsterdam_now + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();
    debug!("tomorrow {}", tomorrow);
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

    debug!("stmt {:?}", stmt);
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
                    date: row.date.to_string(),
                    time: parse_gtfs_time(&row.departure_time),
                    stop_name: row.stop_name.to_string(),
                })
                .collect_vec();

            rest_stops.pop();

            models::ListItem {
                date: active_stop.date.to_string(),
                time: parse_gtfs_time(&active_stop.departure_time),
                rest_stops,
                end_stop: models::ListItemStop {
                    date: last.date.to_string(),
                    time: parse_gtfs_time(&last.departure_time),
                    stop_name: last.stop_name.to_string(),
                },
            }
        })
        .sorted_by_key(|list_item| (list_item.date.to_owned(), list_item.time.to_owned()))
        .collect_vec();

    list_items.truncate(64);

    let stop_name: String = conn
        .query_row(
            "select stop_name from stops where stop_id = ?;",
            &[&sid],
            |row| row.get(0),
        )
        .unwrap();

    let feed_info = get_feed_info(&conn);

    let context = models::MainCtx {
        content: models::Content::DeparturesCtx { list_items },
        title: format!("Van {}", stop_name),
        feed_info,
        download_date: fs::read_to_string("/data/download_date").ok(),
    };

    Template::render("upcoming-departures", &context)
}

#[get("/public/<file..>")]
fn public(file: PathBuf) -> Option<CachedFile> {
    NamedFile::open(Path::new("public/").join(file))
        .ok()
        .map(|nf| CachedFile(nf))
}

// NOTE Service_worker needs to be hosted from root
// NOTE Not hard coding the path, otherwise recompile is needed when changing sw file name
//      This does mean that everything in ./public/scripts is hosted at `/`, but we don't care.
#[get("/<sw>")]
fn service_worker(sw: &RawStr) -> Option<CachedFile> {
    NamedFile::open(Path::new("public").join("scripts").join(sw.as_str()))
        .ok()
        .map(|nf| CachedFile(nf))
}

fn main() {
    pretty_env_logger::init();

    let routes = routes![index, upcoming_departures, public, service_worker];

    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes)
        .attach(Template::fairing())
        .launch();
}
