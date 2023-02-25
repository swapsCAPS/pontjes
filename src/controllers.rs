use chrono::Utc;
use chrono_tz::Europe::Amsterdam;
use itertools::Itertools;
use rocket_sync_db_pools::rusqlite;
use std::fs;

use crate::models::{Content, ListItem, ListItemStop, MainCtx, Row, Stop};
use crate::queries;
use crate::utils::{get_feed_info, PontjesDb};

pub async fn index(db: PontjesDb) -> Result<MainCtx, rusqlite::Error> {
    db.run(|conn| {
        let mut stmt = conn.prepare(queries::INDEX)?;

        let stops = stmt
            .query_map(rusqlite::params![], |row| Ok(Stop {
                stop_id: row.get(0)?,
                stop_name: row.get(1)?,
            }))?
            .map(|x| x.unwrap())
            .collect_vec();

        let feed_info = get_feed_info(&conn)?;

        Ok(MainCtx {
            page_title: String::from("pont.app"),
            page_description: String::from("Je appie voor de Amsterdamse pont tijden. Elke dag een vers geïmporteerde GVB dienstregeling, dus zo snel mogelijk up to date."),
            title: String::from("Vanaf"),
            feed_info: Some(feed_info),
            download_date: fs::read_to_string("/data/download_date").ok(),
            content: Some(Content::IndexCtx { stops }),
        })
    }).await
}

pub async fn upcoming_departures(db: PontjesDb, sid: String) -> Result<MainCtx, rusqlite::Error> {
    db.run(move |conn| {
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
            .prepare(queries::DEPARTURES)?
            .query_map(
                &[
                (":today", &today),
                (":tomorrow", &tomorrow),
                (":sid", &sid),
                (":time", &time),
                ],
                |row| Ok(Row {
                    date: row.get(0)?,
                    departure_time: row.get(1)?,
                    stop_name: row.get(2)?,
                    stop_sequence: row.get(3)?,
                    stop_id: row.get(4)?,
                    trip_id: row.get(5)?,
                }),
            )?
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
            // We might get trips that do not contain our stop. Not sure why, but sometimes we
            // panic.
            // TODO write test!
            // TODO monitor logs to check if this .filter() helped!
            .filter(|row| row.iter().find(|x| x.stop_id == sid).is_some())
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
                    list_item.end_stop.stop_name.to_owned(),
                )
            })
            .collect_vec();

        list_items.truncate(64);

        let stop_name: String = conn
            .query_row(
                "select stop_name from stops where stop_id = ?;",
                &[&sid],
                |row| row.get(0)
            )?;

        let feed_info = get_feed_info(&conn)?;

        Ok(MainCtx {
            page_title: format!("pont.app - {}", &stop_name),
            page_description: format!("{} pont tijden. Elke dag een vers geïmporteerde GVB dienstregeling, dus zo snel mogelijk up to date.", &stop_name),
            content: Some(Content::DeparturesCtx { list_items }),
            title: format!("Vanaf {}", stop_name),
            feed_info: Some(feed_info),
            download_date: fs::read_to_string("/data/download_date").ok(),
        })
    }).await
}
