FROM debian:buster

ENV TZ=Europe/Amsterdam
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt install -y curl unzip libsqlite3-dev sqlite3

COPY target/release/pontjes Rocket.toml /
COPY templates /templates
COPY public /public
COPY data/pontjes_db /data/pontjes_db

EXPOSE 6376

ENTRYPOINT [ "/pontjes" ]
