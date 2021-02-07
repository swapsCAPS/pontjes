FROM debian:buster-20201012-slim

WORKDIR /

# Setup os
ENV TZ=Europe/Amsterdam
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt update && apt install -y sqlite3 curl gcc

# Install rust
# NOTE we need to add "-y" as an arg. while we're at it set default toolchain to nightly for rocket
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y

# Copy source
COPY Cargo.lock Cargo.toml /
COPY src /src
COPY Rocket.toml /
COPY templates /templates
COPY public /public

# Build app
# NOTE if we don't build in the container we might get stupid linker errors
ARG CACHEBUST=1
RUN $HOME/.cargo/bin/cargo build --release
RUN mv target/release/pontjes /

EXPOSE 6376

CMD [ "/pontjes" ]
