db-migrate:
  echo "Migrating ..."
  sqlx migrate run;

db-reset:
  echo "Resetting ..."
  sqlx database drop && sqlx database create && sqlx migrate run
  sqlite3 db.db < seeds/seed-users.sql