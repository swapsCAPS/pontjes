select
  route_long_name,
  date,
  departure_time,
  stop_id,
  stop_name,
  stop_sequence,
  trip_id,
  group_concat(stop_id),
  group_concat(stop_name),
  group_concat(departure_time)
from pont_trips
where
  (
    (date=20200922 and departure_time>"20:00") or date=20200923
  ) and trip_id in (
      select trip_id
      from gvb_stop_times as s
      where s.stop_id=1522966
    )
group by date, trip_id
order by date, departure_time
