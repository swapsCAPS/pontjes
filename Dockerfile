FROM ubuntu:18.04

ENV TZ=Europe/Amsterdam
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt install -y curl unzip libsqlite3-dev sqlite3

COPY import.sql /
COPY init_tables.sql /
COPY full-import.sh /
COPY download-and-import.sh /
COPY target/release/pontjes Rocket.toml /
COPY templates /templates
COPY public /public

EXPOSE 6376

ENTRYPOINT [ "/pontjes" ]
