#!/usr/bin/env sh

set -o errexit
set -o nounset

cmd="$*"

redis_ready () {
  # Check that postgres is up and running on port `5432`:
  dockerize -wait 'tcp://redis:6379' -timeout 5s
}

# We need this line to make sure that this container is started
# after the one with postgres:
until redis_ready; do
  >&2 echo 'Redis is unavailable - sleeping'
done

# It is also possible to wait for other services as well: redis, elastic, mongo
>&2 echo 'Redis is up - continuing...'

# Evaluating passed command (do not touch):
# shellcheck disable=SC2086
exec $cmd
