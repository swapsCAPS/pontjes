pub const INDEX: &str = "
    select distinct stop_name from routes as r
    inner join trips as t on t.route_id = r.route_id
    inner join stop_times as st on st.trip_id = t.trip_id
    inner join stops as s on s.stop_id = st.stop_id
    where agency_id = 'GVB' and r.route_type = 4
    order by stop_name;
";

// Note this was initially done using stop_id to query, but there have been GTFS datasets where the
// same stop had multiple stop_ids. I.e. there were multiple instances of the same stop in the
// dataset. Even worse, you would get a stop called NDSM and a stop called NDSM-werf...
// To work around this I decided to go with stop_name as query input instead. (And add some custom
// sanitization during GTFS import...)
// As you can maybe deduce from my tone, I'm not happy with this.
pub const DEPARTURES: &str = "
    select
        date,
        departure_time,
        stop_name,
        stop_sequence,
        s.stop_id,
        t.trip_id
    from trips as t
    inner join stop_times as st on st.trip_id=t.trip_id
    inner join stops as s on s.stop_id=st.stop_id
    inner join calendar_dates as cd on cd.service_id=t.service_id
    where
        (
            (date = :today and departure_time >= :time) or date = :tomorrow
        ) and t.trip_id in (
            select distinct st.trip_id
            from stop_times as st
            inner join stops as _s on _s.stop_id=st.stop_id
            where _s.stop_name = :stop_name
        )
    order by date, departure_time, stop_name;
";
