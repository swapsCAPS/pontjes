SELECT
  `calendar_dates`.`date`,
  `stop_times`.`departure_time`,
  `stops`.`stop_name`,
  `stop_times`.`stop_id`,
  `trips`.`trip_id`,
  `stop_times`.`stop_sequence`
FROM
  (
    (
      (
        `trips`
        INNER JOIN `calendar_dates` ON (
          `calendar_dates`.`date` = "20201018"
          OR `calendar_dates`.`date` = "20201019"
        )
        AND `calendar_dates`.`service_id` = `trips`.`service_id`
      )
      INNER JOIN `stop_times` ON `stop_times`.`trip_id` = `trips`.`trip_id`
    )
    INNER JOIN `stops` ON `stops`.`stop_id` = `stop_times`.`stop_id`
  )
WHERE
  `trips`.`trip_id` IN (
    SELECT
      `stop_times`.`trip_id`
    FROM
      `stop_times`
    WHERE
      `stop_times`.`stop_id` = 1522966
  )
ORDER BY
  `calendar_dates`.`date`,
  `stop_times`.`departure_time`
