#[macro_use]
extern crate rocket_contrib;
pub mod models;
use rocket_contrib::databases::rusqlite;

#[database("pontjes_db")]
pub struct PontjesDb(rusqlite::Connection);

pub fn get_requested_stop(datum: &Vec<models::Row>, sid: &str) -> String {
    let optional = datum.iter().find(|x| x.stop_id == sid);
    match optional {
        Some(p) => format!("{}{}", p.date, p.departure_time),
        None => String::from("zzz"),
    }
}

// Stupid GVB dataset contains >24:00 times (like 25:00)
pub fn parse_gtfs_time(departure_time: &str) -> String {
    let split: Vec<&str> = departure_time.split(':').collect();

    let parsed_hours = split[0].parse::<i8>().expect(&format!(
        "Could not parse hours from: '{}'!",
        departure_time
    ));

    let fixed_hours = parsed_hours % 24;

    format!("{:02}:{}", fixed_hours, split[1])
}

pub fn get_feed_info(conn: &PontjesDb) -> models::FeedInfo {
    conn.prepare("select * from feed_info limit 1;")
        .unwrap()
        .query_map(&[], |row| models::FeedInfo {
            feed_start_date: row.get(4),
            feed_end_date: row.get(5),
            feed_version: row.get(6),
        })
        .unwrap()
        .nth(0)
        .expect("Did not get feed info!")
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_gtfs_time() {
        assert_eq!("00:30", parse_gtfs_time("00:30"));
        assert_eq!("12:30", parse_gtfs_time("12:30"));
        assert_eq!("00:30", parse_gtfs_time("24:30"));
        assert_eq!("03:30", parse_gtfs_time("27:30"));
        assert_eq!("00:12", parse_gtfs_time("0:12"));
    }
}
