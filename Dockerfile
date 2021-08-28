# This first section runs the heavy build operation for the authentication
# server. If we didn't do it this way the final image woould be almost 2GB
# instead of its current <100mb size
FROM rust:1.54 as builder

RUN USER=root cargo new --bin authy
WORKDIR ./authy
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY . .

RUN rm ./target/release/deps/authy*
RUN cargo build --release

# This is the image that actually gets pushed to docker hub. It contains
# only the executable that needs to be run for the server
FROM debian:buster-slim
ARG APP=/usr/src/authy

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*


ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /authy/target/release/authy ${APP}/authy

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
EXPOSE 8000

CMD ["./authy"]

