set dotenv-load


init:
	cargo install cargo-watch
	cargo install sqlx-cli
	sqlx database create
	just db-migrate

dev-server:
	cargo watch -w src -w templates -w tailwind.config.js -w input.css -x run 

dev-tailwind:
	./tailwindcss -i input.css -o assets/output.css --watch=always

build-server:
	cargo build --release

build-tailwind:
	./tailwindcss -i input.css -o assets/output.css --minify


db-migrate:
  echo "Migrating ..."
  sqlx migrate run --source $MIGRATIONS_PATH;

db-reset:
  echo "Resetting ..."
  sqlx database drop && sqlx database create && sqlx migrate run --source $MIGRATIONS_PATH
  sqlite3 $DATABASE_PATH < seeds/seed-users.sql

dev:
	#!/bin/sh
	just dev-tailwind &
	pid1=$!
	just dev-server &
	pid2=$!
	trap "kill $pid1 $pid2" EXIT
	wait $pid1 $pid2
