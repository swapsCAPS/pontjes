FROM rustlang/rust:nightly-buster-slim AS builder

WORKDIR /

COPY Cargo.lock Cargo.toml /
COPY src /src
COPY scripts /scripts
RUN cargo build --release
RUN scripts/download-and-import.sh

# ---

FROM rustlang/rust:nightly-buster-slim
ENV TZ=Europe/Amsterdam
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt update
RUN apt install -y curl unzip libsqlite3-dev sqlite3

COPY Rocket.toml /
COPY templates /templates
COPY public /public

# Built in prev stage
COPY --from=0 target/release/pontjes /
COPY --from=0 data/pontjes_db /data/pontjes_db

EXPOSE 6376

ENTRYPOINT [ "/pontjes" ]
