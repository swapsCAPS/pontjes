use rocket::fs::NamedFile;
use rocket::serde::Serialize;

use crate::utils;

#[derive(Serialize, Debug)]
pub struct Stop {
    pub stop_name: String,
}

#[derive(Serialize, Debug)]
pub struct Row {
    pub date: String,
    pub departure_time: String,
    pub stop_name: String,
    pub stop_id: String,
    pub trip_id: String,
    pub stop_sequence: String,
}

#[derive(Serialize)]
pub struct ListItemStop {
    pub date: String,
    pub time: String,
    pub date_time: String,
    pub stop_name: String,
}

// TODO implement std::cmp::Ordering
impl ListItemStop {
    pub fn new(date: &str, time: &str, stop_name: &str) -> ListItemStop {
        let (date, time) = utils::gtfs_to_sane_date(&date, &time);
        ListItemStop {
            date: date.to_owned(),
            time: time.to_owned(),
            date_time: format!("{}{}", &date, &time),
            stop_name: stop_name.to_owned(),
        }
    }

    pub fn from(row: &Row) -> ListItemStop {
        let (date, time) = utils::gtfs_to_sane_date(&row.date, &row.departure_time);
        ListItemStop {
            date: date.to_owned(),
            time: time.to_owned(),
            date_time: format!("{}{}", &date, &time),
            stop_name: row.stop_name.to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct ListItem {
    pub start_stop: ListItemStop,
    pub rest_stops: Vec<ListItemStop>,
    pub end_stop: ListItemStop,
}

#[derive(Serialize)]
pub enum Content {
    IndexCtx { stops: Vec<Stop> },
    DeparturesCtx { list_items: Vec<ListItem> },
}

#[derive(Serialize)]
pub struct MainCtx {
    pub page_title: String,
    pub page_description: String,
    pub feed_info: Option<FeedInfo>,
    pub title: String,
    pub download_date: Option<String>,
    pub content: Option<Content>,
}

#[derive(Serialize)]
pub struct ErrorCtx {
    pub msg: String,
}

#[derive(Serialize)]
pub struct FeedInfo {
    pub feed_start_date: String,
    pub feed_end_date: String,
    pub feed_version: String,
}

pub struct CachedFile(pub NamedFile);

impl<'r> rocket::response::Responder<'r, 'r> for CachedFile {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'r> {
        rocket::Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", "max-age=86400")
            .ok()
    }
}
