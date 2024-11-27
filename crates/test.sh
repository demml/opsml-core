#!/bin/bash

# Set default values if environment variables are not set
: "${OPSML_STORAGE_URI:=default_storage_uri}"
: "${OPSML_SERVER_PORT:=3000}"
: "${TEST_NAME:=test_filesystemstorage_with_http_google}"

export OPSML_STORAGE_URI
export PORT=$OPSML_SERVER_PORT

# Start the opsml_server in the background
cargo run opsml_server &

# Capture the server PID to kill it later
SERVER_PID=$!

# Ensure the server is killed on script exit
trap "kill $SERVER_PID" EXIT

# get the host and port

# Wait for the server to start (adjust the sleep time as needed)
sleep 5

export OPSML_TRACKING_URI="http://localhost:$PORT"

# Run the tests
cargo test -p opsml-storage $TEST_NAME -- --nocapture
