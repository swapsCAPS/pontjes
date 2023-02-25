use chrono::{Duration, NaiveDate};
use rocket_sync_db_pools::{database, rusqlite};
use std::{error::Error, fmt::Display};

use crate::models;

#[database("rusqlite")]
struct Db(rusqlite::Connection);

#[database("pontjes_db")]
pub struct PontjesDb(rusqlite::Connection);

pub fn get_requested_stop(datum: &Vec<models::Row>, sid: &str) -> String {
    let optional = datum.iter().find(|x| x.stop_id == sid);
    match optional {
        Some(p) => format!("{}{}", p.date, p.departure_time),
        None => String::from("zzz"),
    }
}

pub fn get_feed_info(conn: &rusqlite::Connection) -> Result<models::FeedInfo, rusqlite::Error> {
    conn.prepare("select * from feed_info limit 1;")?
        .query_map(rusqlite::params![], |row| {
            Ok(models::FeedInfo {
                feed_start_date: row.get(4)?,
                feed_end_date: row.get(5)?,
                feed_version: row.get(6)?,
            })
        })?
        .nth(0)
        .expect("Expected at least 1 feed_info")
}

// TODO move this to method on ListItem or smth
// TODO probably return DateTime objects
pub fn gtfs_to_sane_date(date: &str, time: &str) -> (String, String) {
    let split: Vec<&str> = time.split(':').collect();

    let parsed_hours = split[0]
        .parse::<i8>()
        .expect(&format!("Could not parse hours from: '{}'!", time));

    let mut d =
        NaiveDate::parse_from_str(date, "%Y%m%d").expect(&format!("Could not parse date {}", date));

    if parsed_hours >= 24 {
        d = d + Duration::days(1);
    }

    let fixed_hours = parsed_hours % 24;

    (
        format!("{}", d.format("%Y%m%d")),
        format!("{:02}:{}", fixed_hours, split[1]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gtfs_to_sane_date() {
        assert_eq!(
            (String::from("20210919"), String::from("00:30")),
            gtfs_to_sane_date("20210919", "00:30")
        );
        assert_eq!(
            (String::from("20210919"), String::from("12:30")),
            gtfs_to_sane_date("20210919", "12:30")
        );
        assert_eq!(
            (String::from("20210920"), String::from("00:30")),
            gtfs_to_sane_date("20210919", "24:30")
        );
        assert_eq!(
            (String::from("20210920"), String::from("03:30")),
            gtfs_to_sane_date("20210919", "27:30")
        );
        assert_eq!(
            (String::from("20210919"), String::from("00:12")),
            gtfs_to_sane_date("20210919", "0:12")
        );
    }
}
