FROM rust:slim-trixie AS builder
RUN apt-get update && apt-get install -y --no-install-recommends libhtsengine1 libclang-dev dpkg-dev && rm -rf /var/lib/apt/lists/*
RUN export LIBDIR="/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)" && ln -sf "$LIBDIR/libHTSEngine.so.1" "$LIBDIR/libHTSEngine.so"
WORKDIR /ws
COPY Cargo.toml /ws/Cargo.toml
COPY Cargo.lock /ws/Cargo.lock
COPY ./data/all.csv /ws/data/all.csv
COPY ./data/symbol.csv /ws/data/symbol.csv
COPY ./crates /ws/crates
RUN cargo build -r --bin synth-server
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends libhtsengine1 && rm -rf /var/lib/apt/lists/*
WORKDIR /ws
COPY --from=builder /ws/target/release/synth-server /ws/synth-server
COPY LICENSE /ws/LICENSE
COPY README.md /ws/README.md
COPY THIRD_PARTY_NOTICES /ws/THIRD_PARTY_NOTICES
ENV SETTING_PATH=/data/config.toml
CMD ["/bin/sh", "-c", "cd / && echo '----README----\n\n' && cat /ws/README.md && echo '\n\n----LICENSE----\n\n' && cat /ws/LICENSE && echo '\n\n----THIRD_PARTY_NOTICES----\n\n' && cat /ws/THIRD_PARTY_NOTICES && echo '\n\n----RUN----\n\n' && /ws/synth-server"]
