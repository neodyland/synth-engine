FROM rust:slim-trixie AS builder
RUN apt-get update && apt-get install -y --no-install-recommends libhtsengine1 libclang-dev dpkg-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /ws
RUN cp "/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libHTSEngine.so.1" "/ws/libHTSEngine.so.1"
RUN cp "/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libgcc_s.so.1" "/ws/libgcc_s.so.1"
RUN ln -sf "/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libHTSEngine.so.1" "/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libHTSEngine.so"
COPY Cargo.toml /ws/Cargo.toml
COPY Cargo.lock /ws/Cargo.lock
COPY ./data/all.csv.zstd /ws/data/all.csv.zstd
COPY ./data/symbol.csv /ws/data/symbol.csv
COPY ./crates /ws/crates
RUN cargo build -r --bin synth-server
FROM busybox:glibc
WORKDIR /ws
COPY --from=builder /ws/target/release/synth-server /ws/synth-server
COPY --from=builder /ws/libHTSEngine.so.1 /usr/lib/libHTSEngine.so.1
COPY --from=builder /ws/libgcc_s.so.1 /usr/lib/libgcc_s.so.1
COPY LICENSE /ws/LICENSE
COPY README.md /ws/README.md
COPY THIRD_PARTY_NOTICES /ws/THIRD_PARTY_NOTICES
ENV SETTING_PATH=/data/config.toml
CMD ["/bin/sh", "-c", "export LD_LIBRARY_PATH=/usr/lib && cd / && echo '----README----\n\n' && cat /ws/README.md && echo '\n\n----LICENSE----\n\n' && cat /ws/LICENSE && echo '\n\n----THIRD_PARTY_NOTICES----\n\n' && cat /ws/THIRD_PARTY_NOTICES && echo '\n\n----RUN----\n\n' && exec /ws/synth-server"]
