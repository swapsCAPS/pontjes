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
pub struct ListItemStop<'a> {
    pub date: &'a str,
    pub time: &'a str,
    pub stop_name: &'a str,
}

#[derive(Serialize)]
pub struct ListItem<'a> {
    pub date: &'a str,
    pub time: &'a str,
    pub rest_stops: Vec<ListItemStop<'a>>,
    pub end_stop: ListItemStop<'a>,
}

#[derive(Serialize)]
pub struct IndexCtx<'a> {
    pub title: &'a str,
    pub stops: Vec<Stop>,
}

#[derive(Serialize)]
pub struct DeparturesCtx<'a> {
    pub title: &'a str,
    pub requested_stop: &'a str,
    pub list_items: Vec<ListItem<'a>>,
}

#[derive(Serialize)]
pub struct ErrorCtx<'a> {
    pub msg: &'a str,
}
