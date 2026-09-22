FROM node:24-bookworm-slim AS web
WORKDIR /src/web
COPY web/package*.json ./
RUN npm ci
COPY web/ ./
RUN npm run check && npm run build

FROM rust:1.90-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY --from=web /src/web/build ./web/build
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl fontconfig \
    fonts-dejavu-core fonts-liberation2 fonts-lato fonts-open-sans fonts-roboto fonts-roboto-slab \
    fonts-firacode fonts-comfortaa fonts-noto-core fonts-noto-extra fonts-urw-base35 \
    fonts-freefont-ttf fonts-cantarell fonts-crosextra-carlito fonts-crosextra-caladea fonts-texgyre \
    fonts-linuxlibertine fonts-sil-gentiumplus fonts-sil-charis fonts-sil-andika \
    fonts-lobster fonts-lobstertwo fonts-yanone-kaffeesatz fonts-cabin fonts-quicksand \
    fonts-league-spartan fonts-league-mono fonts-radisnoir fonts-junicode fonts-gfs-didot fonts-gfs-neohellenic \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/coverforge /usr/local/bin/coverforge
COPY --from=web /src/web/build /app/web/build
COPY templates /app/default-templates
EXPOSE 3099
ENTRYPOINT ["/usr/local/bin/coverforge"]
