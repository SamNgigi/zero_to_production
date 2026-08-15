#!/usr/bin/env bash
set -x # Print commands as they run
set -eo pipefail # Exit on Error & Catch Pipe Failures as well


# Check if a custom redis name has been set, otherwise default to 'newsletter'
DB_NAME="${REDIS_DB:=newsletter}"

# Allow to skip Docker if a Dockerized Redis instance is already running
if [[ -z "${SKIP_DOCKER}" ]] # -z Zero length
then
  # If a redis container is running, print instructions to kill it and exit.
  RUNNING_REDIS_CONTAINER=$(docker ps --filter 'name=redis' --format '{{.ID}}')
  if [[ -n $RUNNING_REDIS_CONTAINER ]]; then # -n Non-Zero Length
    echo >&2 "There is a redis container already running, kill it with"
    echo >&2 "  docker kill ${RUNNING_CONTAINER}"
    exit 1
  fi

  # Lauch Redis using Docker
  docker run \
    -p "6379:6379" \
    -d \
    --name "${DB_NAME}_axum_redis" \
    redis:8
fi

>&2 echo "Redis is ready to go!"
