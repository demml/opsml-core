#!/bin/bash

export OPSML_STORAGE_URI=${OPSML_STORAGE_URI}

# Start the opsml_server in the background
cargo run opsml_server &

# Capture the server PID to kill it later
SERVER_PID=$!

# Ensure the server is killed on script exit
trap "kill $SERVER_PID" EXIT

# get the host and port

# Wait for the server to start (adjust the sleep time as needed)
sleep 5

export OPSML_TRACKING_URI="http://localhost:3000"

# Run the tests
TEST_NAME=${TEST_NAME:-test_filesystemstorage_with_http_google}
cargo test -p opsml-storage $TEST_NAME -- --nocapture
