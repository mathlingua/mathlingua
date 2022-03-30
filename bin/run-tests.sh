#!/usr/bin/env bash

# stop on first error
set -e

# the port to start the backend on (default 8090)
RESOLVED_SERVER_PORT=${SERVER_PORT:-8090}
RESOLVED_CLIENT_PORT=${CLIENT_PORT:-${RESOLVED_SERVER_PORT}}
ROOT_DIR="$(pwd)"

if [ -z "${NO_BUILD}" ]
then
  # rebuild the MathLingua code
  echo "Rebuilding the frontend and backend code..."
  ./bin/build-assets.sh
fi

# enter the directory to the root of the webapp
cd src/main/webapp

if [ -z "${NO_SERVER}" ]
then
  # open the editor in the background so the tests can run
  cd cypress-codex
  echo "Starting the backend on port ${RESOLVED_SERVER_PORT}..."
  java -jar ../../../../build/releases/mathlingua-*.jar edit --no-open --port "${RESOLVED_SERVER_PORT}" &
  cd ..
fi

# allow the tests to fail so we can shutdown the server if the tests fail
set +e

# run the tests
echo "Running the tests..."
./node_modules/.bin/cypress "${1:-run}" --env PORT="${RESOLVED_CLIENT_PORT}"
EXIT_CODE=$?

if [ -z "${NO_SERVER}" ]
then
  # shutdown the server
  echo "Shutting down the backend..."
  curl "http://localhost:${RESOLVED_SERVER_PORT}/api/shutdown"
fi

# if the cypress tests exited with an exit code other then 0, fail the tests
if [ "${EXIT_CODE}" -ne 0 ]
then
  exit "${EXIT_CODE}"
fi

# again fail on error
set -e

# switch back to the project root
cd "${ROOT_DIR}"

echo "Running the backend tests..."
mvn test

echo "Running the component tests..."
cd src/main/webapp
npm test -- --watchAll=false
