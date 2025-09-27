FROM rust:1.85.0-bookworm AS builder
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder ./target/release/cmd /husni-portfolio
COPY ./statics /statics
COPY ./version.json /version.json
CMD ["/husni-portfolio"]
