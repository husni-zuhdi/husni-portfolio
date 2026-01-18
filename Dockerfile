FROM rust:1.89.0-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian13:latest-amd64
COPY --from=builder /app/target/release/husni-portfolio /husni-portfolio
COPY ./statics /statics
COPY ./version.json /version.json
CMD ["/husni-portfolio"]
