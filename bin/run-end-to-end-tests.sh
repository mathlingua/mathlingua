#!/usr/bin/env bash

# stop on first error
set -e

# rebuild the MathLingua code
echo "Rebuilding the frontend and backend code..."
./bin/build-assets.sh

# enter the directory that contains the MathLingua files used in the tests
cd src/main/webapp/cypress-codex

# open the editor in the background so the tests can run
echo "Starting the backend..."
java -jar ../../../../build/releases/mathlingua-*.jar edit --no-open --port 8090 &

# allow the tests to fail so we can shutdown the server if the tests fail
set +e

# run the tests
echo "Running the tests..."
cd ..
./node_modules/.bin/cypress open
EXIT_CODE=$?

# shutdown the server
echo "Shutting down the backend..."
curl http://localhost:8090/api/shutdown

# exit with the exit code of the cypress tests
exit ${EXIT_CODE}
