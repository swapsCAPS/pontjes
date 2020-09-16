drop table gvb_routes;
create table gvb_routes as
select * from routes where agency_id='GVB';

drop table gvb_ferries;
create table gvb_ferries as
select * from gvb_routes where route_url like '%veerboot%';

drop table pontjes;
create table pontjes as
select
  route_long_name,
  date,
  departure_time,
  stop_name,
  trip_headsign,
  t.trip_id,
  stop_sequence
from gvb_ferries as f
inner join trips as t on f.route_id=t.route_id
inner join calendar_dates as cd on cd.service_id=t.service_id
inner join stop_times as st on st.trip_id=t.trip_id
inner join stops as s on s.stop_id=st.stop_id
order by route_long_name, date, departure_time;
