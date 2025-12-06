FROM rust:1.89.0-bookworm AS builder
COPY . .
RUN cargo build --release --locked

FROM gcr.io/distroless/cc
COPY --from=builder ./target/release/cmd /husni-portfolio
COPY ./statics /statics
COPY ./version.json /version.json
CMD ["/husni-portfolio"]
