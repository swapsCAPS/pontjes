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
use rocket::fs::NamedFile;
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::rusqlite::params;
use std::fs;
use std::path::{Path, PathBuf};

pub mod models;
pub mod queries;
pub mod utils;

use models::{Content, ListItem, ListItemStop, MainCtx, Row, Stop};
use utils::{get_feed_info, PontjesDb};

lazy_static! {
    static ref DOWNLOAD_DATE: Option<String> = fs::read_to_string("/data/download_date").ok();
}

#[get("/")]
async fn index(db: PontjesDb) -> Template {
    let context = db.run(|conn| {
        let mut stmt = conn
            .prepare(queries::INDEX)
            .unwrap();

        let stops = stmt
            .query_map(params![], |row| Ok(Stop {
                stop_id: row.get(0).unwrap(),
                stop_name: row.get(1).unwrap(),
            }))
        .unwrap()
            .map(|x| x.unwrap())
            .collect_vec();

        let feed_info = get_feed_info(&conn);

        MainCtx {
            page_title: String::from("pont.app"),
            page_description: String::from("Je appie voor de Amsterdamse pont tijden. Elke dag een vers geïmporteerde GVB dienstregeling, dus zo snel mogelijk up to date."),
            title: String::from("Vanaf"),
            feed_info,
            download_date: fs::read_to_string("/data/download_date").ok(),
            content: Content::IndexCtx { stops },
        }
    }).await;

    Template::render("index", context)
}

#[get("/upcoming-departures/<raw_sid>")]
async fn upcoming_departures(db: PontjesDb, raw_sid: &str) -> Template {
    let sid = raw_sid.to_string();
    let context = db.run(move |conn| {
        let now = Utc::now();
        let amsterdam_now = now.with_timezone(&Amsterdam);
        let today = amsterdam_now.format("%Y%m%d").to_string();
        let tomorrow = (amsterdam_now + chrono::Duration::days(1))
            .format("%Y%m%d")
            .to_string();
        let time = amsterdam_now.format("%H:%M").to_string();

        debug!("now {}", now);
        debug!("amsterdam_now {}", amsterdam_now);
        debug!("today {}", today);
        debug!("tomorrow {}", tomorrow);

        let results = conn
            .prepare(queries::DEPARTURES)
            .unwrap()
            .query_map(
                &[
                (":today", &today),
                (":tomorrow", &tomorrow),
                (":sid", &sid),
                (":time", &time),
                ],
                |row| Ok(Row {
                    date: row.get(0).unwrap(),
                    departure_time: row.get(1).unwrap(),
                    stop_name: row.get(2).unwrap(),
                    stop_sequence: row.get(3).unwrap(),
                    stop_id: row.get(4).unwrap(),
                    trip_id: row.get(5).unwrap(),
                }),
            )
            .unwrap()
            .map(|x| x.unwrap())
            .collect_vec();

        let tuples: Vec<(String, Row)> = results
            .into_iter()
            .map(|r| (format!("{}{}", r.date, r.trip_id), r))
            .collect_vec();

        let group_map = tuples.into_iter().into_group_map();

        let mut list_items: Vec<ListItem> = group_map
            .values()
            // TODO The length filter is prolly too naive
            .filter(|row| row.len() > 1 && row[row.len() - 1].stop_id != sid)
            .map(|trip| {
                let active_stop: &Row = trip.iter().find(|x| x.stop_id == sid).unwrap();
                let last: &Row = &trip[trip.len() - 1];

                let start_stop = ListItemStop::from(&active_stop);
                let end_stop = ListItemStop::from(&last);

                let mut rest_stops: Vec<ListItemStop> = trip
                    .iter()
                    .map(|row| ListItemStop::from(row))
                    .filter(|lis| {
                        lis.stop_name != active_stop.stop_name && lis.date_time > start_stop.date_time
                    })
                .collect_vec();

                rest_stops.pop();

                ListItem {
                    start_stop,
                    rest_stops,
                    end_stop,
                }
            })
            .sorted_by_key(|list_item| {
                (
                    list_item.start_stop.date.to_owned(),
                    list_item.start_stop.time.to_owned(),
                )
            })
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

        MainCtx {
            page_title: format!("pont.app - {}", &stop_name),
            page_description: format!("{} pont tijden. Elke dag een vers geïmporteerde GVB dienstregeling, dus zo snel mogelijk up to date.", &stop_name),
            content: Content::DeparturesCtx { list_items },
            title: format!("Vanaf {}", stop_name),
            feed_info,
            download_date: fs::read_to_string("/data/download_date").ok(),
        }
    }).await;

    Template::render("upcoming-departures", context)
}

#[get("/public/<file..>")]
async fn public(file: PathBuf) -> Option<models::CachedFile> {
    NamedFile::open(Path::new("public/").join(file))
        .await
        .ok()
        .map(|nf| models::CachedFile(nf))
}

// NOTE Service_worker needs to be hosted from root
// NOTE Not hard coding the path, otherwise recompile is needed when changing sw file name
//      This does mean that everything in ./public/scripts is hosted at `/`, but we don't care.
#[get("/<sw>")]
async fn service_worker(sw: &str) -> Option<models::CachedFile> {
    NamedFile::open(Path::new("public").join("scripts").join(sw))
        .await
        .ok()
        .map(|nf| models::CachedFile(nf))
}

#[launch]
fn rocket() -> _ {
    pretty_env_logger::init();

    let routes = routes![index, upcoming_departures, public, service_worker];

    rocket::build()
        .attach(PontjesDb::fairing())
        .attach(Template::fairing())
        .mount("/", routes)
}
