use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Europe::Amsterdam;
use rocket_sync_db_pools::{database, rusqlite};

use crate::models;

#[database("rusqlite")]
struct Db(rusqlite::Connection);

#[database("pontjes_db")]
pub struct PontjesDb(rusqlite::Connection);

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
        .unwrap_or_else(|| {
            warn!("Should not happen! No feed_info found!");
            // TODO Box or wrap our errors... Dont return lib errors as our own...
            Err(rusqlite::Error::QueryReturnedNoRows)
        })
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

/**
 * Returns a NaiveDateTime for the given `Some(&str)`.
 * If `None` is passed or parsing goes wrong a NaiveDateTime for the current Amsterdam time is
 * returned
 */
pub fn parse_date_time(dt: Option<&str>) -> NaiveDateTime {
    match dt {
        Some(dt) => NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M")
            .unwrap_or_else(|_| Utc::now().with_timezone(&Amsterdam).naive_local()),
        None => Utc::now().with_timezone(&Amsterdam).naive_local(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parse_date_time_1() {
        let result = parse_date_time(None);
        println!("{}", result)
    }

    #[test]
    fn it_parse_date_time_2() {
        let result = parse_date_time(Some("2022-01-01T00:00"));
        println!("{}", result)
    }

    #[test]
    fn it_parse_date_time_3() {
        let result = parse_date_time(Some("2022-06-01T00:00"));
        println!("{}", result)
    }

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
