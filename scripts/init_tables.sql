--- Clean up DB a bit so we have less to query. NOTE the order of execution matters.
delete from trips where trip_id not in (
  select distinct t.trip_id from trips as t
  inner join routes as r on r.route_id = t.route_id
  where r.agency_id = 'GVB' and r.route_type = 4
);

delete from stops where stop_id not in (
  select distinct st.stop_id from stop_times as st
  inner join trips  as t on t.trip_id  = st.trip_id
  inner join routes as r on r.route_id = t.route_id
  where r.agency_id = 'GVB' and r.route_type = 4
);

delete from stop_times where stop_id not in (
  select distinct stop_id from stops
);

--- Remove prefixes
update stops set stop_name = replace(stop_name, 'Amsterdam, ', '');
update stops set stop_name = replace(stop_name, 'Velsen, ', '');
update stops set stop_name = replace(stop_name, 'Spaarndam, ', '');
update stops set stop_name = replace(stop_name, 'Assendelft, ', '');

--- Remove seconds from departure_time
update stop_times set departure_time = substr(departure_time, 1, 5);

--- Magic to ensure we only have one CS entry. NOTE the order of execution matters.
--- Set all CS stop_times first.
update stop_times set stop_id = "000001"
where stop_id in (
  select distinct s.stop_id from routes as r
  inner join trips      as t  on t.route_id = r.route_id
  inner join stop_times as st on st.trip_id = t.trip_id
  inner join stops      as s  on s.stop_id  = st.stop_id
  where agency_id   = 'GVB'              and
        s.stop_name = "Centraal Station" and
        r.route_type = 4
);

--- Then remove all CS stops and create a new single entry
delete from stops where stop_name = "Centraal Station";
insert into stops (stop_id, stop_name) values ("000001", "Centraal Station");
