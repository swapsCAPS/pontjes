.mode csv
delete from stops;
delete from calendar_dates;
delete from trips;
delete from stop_times;
delete from routes;
.import gtfs/stops.txt stops
.import gtfs/calendar_dates.txt calendar_dates
.import gtfs/trips.txt trips
.import gtfs/stop_times.txt stop_times
.import gtfs/routes.txt routes

create unique index if not exists stops_stop_id on stops (stop_id);

create index if not exists cd_service_id on calendar_dates (service_id);

create unique index if not exists trips_trip_id on trips (trip_id);
create index if not exists trips_route_id on trips (route_id);
create index if not exists trips_service_id on trips (service_id);

create index if not exists st_trip_id on stop_times (trip_id);
create index if not exists st_stop_id on stop_times (stop_id);

create unique index if not exists routes_route_id on routes (route_id);
create index if not exists routes_agency_id on routes (agency_id);
create index if not exists routes_route_url on routes (route_url);
