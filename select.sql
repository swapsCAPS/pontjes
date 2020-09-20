select
  route_long_name,
  date,
  departure_time,
  stop_id,
  stop_name,
  stop_sequence,
  trip_id
from pont_trips
where
  (
    (date=20200920 and departure_time>"20:50") or date=20200921
  ) and trip_id in (
      select trip_id
      from gvb_stop_times as s
      where s.stop_id=1522966
    )
order by date, departure_time
limit 30
