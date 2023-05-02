####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN update-ca-certificates

# Create appuser
ENV USER=runner
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /app

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM debian:buster-slim

RUN apt-get update && apt-get install -y libncurses-dev

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Use an unprivileged user.
USER runner:runner

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/client /app/client
COPY --from=builder /app/target/release/server /app/server

ENV PATH="/app:${PATH}"

# Copy config
COPY ./config/vocab.txt /app/config/vocab.txt

EXPOSE 22345/tcp

CMD ["server"]
