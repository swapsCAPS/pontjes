use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Stop {
    pub stop_id: String,
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
    pub raw_time: String,
    pub stop_name: String,
}

#[derive(Serialize)]
pub struct ListItem {
    pub date: String,
    pub time: String,
    pub raw_time: String,
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
    pub feed_info: FeedInfo,
    pub title: String,
    pub download_date: Option<String>,
    pub content: Content,
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
