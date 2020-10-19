update stops set stop_name = replace(stop_name, 'Amsterdam, ', '');
update stops set stop_name = replace(stop_name, 'Velsen, ', '');
update stops set stop_name = replace(stop_name, 'Spaarndam, ', '');
update stops set stop_name = replace(stop_name, 'Assendelft, ', '');

update stop_times set departure_time = substr(departure_time, 1, 5);

