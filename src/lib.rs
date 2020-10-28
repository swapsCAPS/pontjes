pub mod models;

pub fn get_requested_stop(datum: &Vec<models::Row>, sid: &str) -> String {
    let optional = datum.iter().find(|x| x.stop_id == sid);
    match optional {
        Some(p) => format!("{}{}", p.date, p.departure_time),
        None => String::from("zzz"),
    }
}
