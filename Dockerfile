FROM rust:1.31

WORKDIR /app
COPY . .

RUN cargo install --path .

EXPOSE 8080
CMD ["npm", "run", "start:prod"]

CMD ["myapp"]
