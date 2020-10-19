select distinct s.stop_id, stop_name from routes as r
inner join trips as t on t.route_id = r.route_id
inner join stop_times as st on st.trip_id = t.trip_id
inner join stops as s on s.stop_id = st.stop_id
where agency_id = "GVB" and r.route_url like "%veerboot%";

