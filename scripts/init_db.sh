#! /usr/bin/env bash

set -x
set -eo pipefail

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD=${PSTGRES_PASSWORD:=password}
# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME=${POSTGRES_DB:=newsletter}
# Check if a custom post has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"
if [[ -z "${SKIP_DOCKER}" ]]
then 
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d \
        postgres \
        postgres -N 1000 # Increased maximum number of connections for testing purpose
fi
sleep 3

>&2 echo "Postgres is up and running on port ${DB_PORT}!"
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, read to go!"
