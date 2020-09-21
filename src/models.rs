use diesel::Queryable;

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Route {
    pub route_id: String,
    pub route_long_name: String,
}

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_code: String,
    pub stop_name: String,
    pub stop_lat: String,
    pub stop_lon: String,
    pub location_type: String,
    pub parent_station: String,
    pub stop_timezone: String,
    pub wheelchair_boarding: String,
    pub platform_code: String,
    pub zone_id: String,
}

#[derive(serde::Serialize, Queryable, Debug)]
pub struct Row {
    pub route_long_name: String,
    pub date: String,
    pub departure_time: String,
    pub stop_name: String,
    pub stop_id: String,
    pub trip_headsign: String,
    pub trip_id: String,
    pub stop_sequence: String,
}
