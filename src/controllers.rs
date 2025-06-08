use chrono::NaiveDateTime;
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
                stop_name: row.get(0)?,
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

pub async fn upcoming_departures(
    db: PontjesDb,
    stop_name: String,
    naive_datetime: NaiveDateTime,
) -> Result<MainCtx, rusqlite::Error> {
    db.run(move |conn| {
        let today = naive_datetime.format("%Y%m%d").to_string();
        let tomorrow = (naive_datetime + chrono::Duration::days(1))
            .format("%Y%m%d")
            .to_string();
        let time = naive_datetime.format("%H:%M").to_string();

        debug!("naive_datetime {}", naive_datetime);
        debug!("today {}", today);
        debug!("tomorrow {}", tomorrow);

        let results = conn
            .prepare(queries::DEPARTURES)?
            .query_map(
                &[
                (":today", &today),
                (":tomorrow", &tomorrow),
                (":stop_name", &stop_name),
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

        println!("results:\n{:?}", results);

        let tuples: Vec<(String, Row)> = results
            .into_iter()
            .map(|r| (format!("{}{}", r.date, r.trip_id), r))
            .collect_vec();

        let group_map = tuples.into_iter().into_group_map();

        println!("group_map:\n{:?}", group_map);

        let mut list_items: Vec<ListItem> = group_map
            .values()
            // TODO The length filter is prolly too naive
            .filter(|row| row.len() > 1 && row[row.len() - 1].stop_name != stop_name)
            // We might get trips that do not contain our stop. Not sure why, but sometimes we
            // panic.
            // TODO write test!
            // TODO monitor logs to check if this .filter() helped!
            .filter(|row| row.iter().find(|x| x.stop_name == stop_name).is_some())
            .map(|trip| {
                println!("trip: {:?}", trip);
                let active_stop: &Row = trip.iter().find(|x| x.stop_name == stop_name).unwrap();
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
                "select stop_name from stops where stop_name = ?;",
                &[&stop_name],
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
