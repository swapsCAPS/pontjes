pub const INDEX: &str = "
    select distinct s.stop_id, stop_name from routes as r
    inner join trips as t on t.route_id = r.route_id
    inner join stop_times as st on st.trip_id = t.trip_id
    inner join stops as s on s.stop_id = st.stop_id
    where agency_id = 'GVB' and r.route_type = 4
    order by stop_name;
";

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
            (date = :today and departure_time > :time) or date = :tomorrow
        ) and t.trip_id in (
            select distinct st.trip_id
            from stop_times as st
            where st.stop_id = :sid
        )
    order by date, departure_time;
";