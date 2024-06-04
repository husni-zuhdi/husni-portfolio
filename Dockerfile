FROM rust:1.73-buster AS builder
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder ./target/release/cmd /husni-portfolio
CMD ["/husni-portfolio"]
