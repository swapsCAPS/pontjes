drop table gvb_ferries;
create table gvb_ferries as
select * from routes
where route_url like '%veerboot%' and agency_id='GVB';

drop table pont_trips;
create table pont_trips as
select
  f.route_id,
  route_long_name,
  route_short_name,
  date,
  departure_time,
  stop_name,
  s.stop_id,
  trip_headsign,
  t.trip_id,
  stop_sequence
from gvb_ferries as f
inner join trips as t on f.route_id=t.route_id
inner join calendar_dates as cd on cd.service_id=t.service_id
inner join stop_times as st on st.trip_id=t.trip_id
inner join stops as s on s.stop_id=st.stop_id
order by route_long_name, date, departure_time;

drop table gvb_stop_times;
create table gvb_stop_times as
select * from stop_times as st
where st.trip_id in (select trip_id from pont_trips);

drop table gvb_stops;
create table gvb_stops as
select * from stops as st
where st.stop_id in (select stop_id from gvb_stop_times);
