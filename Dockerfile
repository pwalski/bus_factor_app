FROM rust:buster as builder
WORKDIR /usr/src/bus_factor_app
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/bus_factor /usr/local/bin/bus_factor
ENTRYPOINT ["bus_factor"]
