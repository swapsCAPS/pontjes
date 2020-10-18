table! {
    routes (route_id) {
        route_id -> Text,
        agency_id -> Text,
        route_short_name -> Text,
        route_long_name -> Text,
        route_desc -> Text,
        route_type -> Text,
        route_color -> Text,
        route_text_color -> Text,
        route_url -> Text,
    }
}
table! {
    stops (stop_id) {
        stop_id -> Text,
        stop_code -> Text,
        stop_name -> Text,
        stop_lat -> Text,
        stop_lon -> Text,
        location_type -> Text,
        parent_station -> Text,
        stop_timezone -> Text,
        wheelchair_boarding -> Text,
        platform_code -> Text,
        zone_id -> Text,
    }
}
table! {
    calendar_dates (service_id) {
        service_id -> Text,
        date -> Text,
        exception_type -> Text,
    }
}
table! {
    trips (trip_id) {
        route_id -> Text,
        service_id -> Text,
        trip_id -> Text,
        realtime_trip_id -> Text,
        trip_headsign -> Text,
        trip_short_name -> Text,
        trip_long_name -> Text,
        direction_id -> Text,
        block_id -> Text,
        shape_id -> Text,
        wheelchair_accessible -> Text,
        bikes_allowed -> Text,
    }
}
table! {
    stop_times (trip_id) {
        trip_id -> Text,
        stop_sequence -> Text,
        stop_id -> Text,
        stop_headsign -> Text,
        arrival_time -> Text,
        departure_time -> Text,
        pickup_type -> Text,
        drop_off_type -> Text,
        timepoint -> Text,
        shape_dist_traveled -> Text,
        fare_units_traveled -> Text,
    }
}
table! {
    gvb_stops (stop_id) {
        stop_id -> Text,
        stop_code -> Text,
        stop_name -> Text,
        stop_lat -> Text,
        stop_lon -> Text,
        location_type -> Text,
        parent_station -> Text,
        stop_timezone -> Text,
        wheelchair_boarding -> Text,
        platform_code -> Text,
        zone_id -> Text,
    }
}
table! {
    gvb_stop_times (stop_id) {
        trip_id -> Text,
        stop_sequence -> Text,
        stop_id -> Text,
        stop_headsign -> Text,
        arrival_time -> Text,
        departure_time -> Text,
        pickup_type -> Text,
        drop_off_type -> Text,
        timepoint -> Text,
        shape_dist_traveled -> Text,
        fare_units_traveled -> Text,
    }
}
table! {
    pont_trips (trip_id) {
      route_long_name -> Text,
      date -> Text,
      departure_time -> Text,
      stop_name -> Text,
      stop_id -> Text,
      trip_headsign -> Text,
      trip_id -> Text,
      stop_sequence -> Text,
    }
}

allow_tables_to_appear_in_same_query!(pont_trips, gvb_stop_times); // sErioUsLy?!
allow_tables_to_appear_in_same_query!(routes, trips, calendar_dates, stop_times, stops); // sErioUsLy?!

