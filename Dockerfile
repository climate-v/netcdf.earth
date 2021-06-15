FROM emscripten/emsdk:1.39.20
MAINTAINER Ugur Cayoglu <cayoglu@me.com>

RUN set -eux && \
    apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    ca-certificates gcc libc6-dev wget build-essential

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
ENV URL="https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
ENV RUST_VERSION=stable


RUN wget "$URL"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

ADD . /source
WORKDIR /source

RUN ./build.sh
FROM node:16
RUN mkdir /root/netcdf
COPY --from=0 /source/earth /root/netcdf/earth
COPY --from=0 /source/web /root/netcdf/web
COPY --from=0 /source/visualize /root/netcdf/visualize
RUN cd /root/netcdf/earth && npm ci
RUN cd /root/netcdf/earth && ./node_modules/.bin/vite build
FROM nginx:alpine
RUN rm -rf /usr/share/nginx/html/*
COPY --from=1 /root/netcdf/earth/dist /usr/share/nginx/html
