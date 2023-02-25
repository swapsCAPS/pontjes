FROM debian:bullseye-20230208-slim

WORKDIR /

# Setup os
ENV TZ=Europe/Amsterdam
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt update && apt install -y sqlite3

COPY bin/pontjes_aarch64-unknown-linux-gnu /pontjes
COPY Rocket.toml /
COPY templates /templates
COPY public /public

CMD [ "/pontjes" ]
