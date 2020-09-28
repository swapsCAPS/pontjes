.mode csv
drop table stops;
drop table calendar_dates;
drop table trips;
drop table stop_times;
drop table routes;
.import gtfs/stops.txt stops
.import gtfs/calendar_dates.txt calendar_dates
.import gtfs/trips.txt trips
.import gtfs/stop_times.txt stop_times
.import gtfs/routes.txt routes
