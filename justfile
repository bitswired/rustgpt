init:
	cargo install cargo-watch
	cargo install sqlx-cli
	sqlx database create

dev-server:
	cargo watch -w src -w templates -w tailwind.config.js -w input.css -x run 

dev-tailwind:
	./tailwindcss -i input.css -o assets/output.css --watch

build-server:
	cargo build --release

build-tailwind:
	./tailwindcss -i input.css -o assets/output.css --minify


db-migrate:
  echo "Migrating ..."
  sqlx migrate run;

db-reset:
  echo "Resetting ..."
  sqlx database drop && sqlx database create && sqlx migrate run
  sqlite3 db.db < seeds/seed-users.sql