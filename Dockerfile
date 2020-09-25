FROM ubuntu:20.04

RUN apt update
RUN apt install -Y libsqlite3-dev sqlite3

COPY import.sql /
COPY init_tables.sql /
COPY full-import.sh /
COPY download-and-import.sh /
COPY target/release/pontjes Rocket.toml /
COPY templates /templates
COPY public /public

EXPOSE 6376

ENTRYPOINT [ "/pontjes" ]
