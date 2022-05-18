FROM rust:1.31

WORKDIR /usr/src/myapp

COPY . .

RUN cargo install --path .

EXPOSE 8080

CMD ["cargo", "run"]

# docker build -t public-chat:latest .
# docker run -d -p 8080:8080 public-chat:latest
