.mode csv
drop table stops;
drop table calendar_dates;
drop table trips;
drop table stop_times;
drop table routes;
.import data/gtfs/stops.txt stops
.import data/gtfs/calendar_dates.txt calendar_dates
.import data/gtfs/trips.txt trips
.import data/gtfs/stop_times.txt stop_times
.import data/gtfs/routes.txt routes
