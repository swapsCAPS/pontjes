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
        where agency_id = 'GVB' and r.route_url like '%veerboot%';
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
        title: "Vertrek van:",
        stops,
    };

    Template::render("index", &context)
}

#[get("/upcoming-departures/<sid>")]
fn stop(conn: PontjesDb, sid: &RawStr) -> Template {
    let mut stmt = conn
        .prepare(
            "
        select distinct s.stop_id, stop_name from routes as r
        inner join trips as t on t.route_id = r.route_id
        inner join stop_times as st on st.trip_id = t.trip_id
        inner join stops as s on s.stop_id = st.stop_id
        where agency_id = 'GVB' and r.route_url like '%veerboot%';
        ",
        )
        .unwrap();

    let stops = stmt
        .query_map(&[&sid.as_str()], |row| models::Stop {
            stop_id: row.get(0),
            stop_name: row.get(1),
        })
        .unwrap()
        .map(|x| x.unwrap())
        .collect_vec();

    let context = models::IndexCtx {
        title: "Vertrek van:",
        stops,
    };

    Template::render("index", &context)
}

fn main() {
    rocket::ignite()
        .attach(PontjesDb::fairing())
        .mount("/", routes![index, stop])
        .mount("/public", routes![cached_files])
        .attach(Template::fairing())
        .launch();
}
