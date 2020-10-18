drop table if exists gvb_ferries;
create table gvb_ferries as
select * from routes
where route_url like '%veerboot%' and agency_id='GVB';

update stops set stop_name = replace(stop_name, 'Amsterdam, ', '');
update stops set stop_name = replace(stop_name, 'Velsen, ', '');
update stops set stop_name = replace(stop_name, 'Spaarndam, ', '');
update stops set stop_name = replace(stop_name, 'Assendelft, ', '');

update stop_times set departure_time = substr(departure_time, 1, 5);

drop table if exists pont_trips;
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

create index if not exists pont_trips_trip_id on pont_trips (trip_id);
create unique index if not exists pont_trips_frequent_query on pont_trips (date, departure_time, trip_id);

drop table if exists gvb_stop_times;
create table gvb_stop_times as
select * from stop_times as st
where st.trip_id in (select trip_id from pont_trips);

create index if not exists gvb_stop_times_stop_id on gvb_stop_times (stop_id);

drop table if exists gvb_stops_temp;
create table gvb_stops_temp as
select * from stops as st
where st.stop_id in (select stop_id from gvb_stop_times);

update gvb_stop_times set stop_id = '00001' where stop_id in (
  select stop_id from gvb_stops_temp where stop_name like '%Centraal%'
);
update gvb_stops_temp  set stop_id = '00001' where stop_name like '%Centraal%';
update pont_trips set stop_id = '00001' where stop_name like '%Centraal%';

drop table if exists gvb_stops;
create table gvb_stops as
select * from gvb_stops_temp group by stop_id;

create unique index if not exists gvb_stops_stop_id on gvb_stops (stop_id);
