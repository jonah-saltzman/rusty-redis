#!/bin/sh
exec cargo run \
    --bin server \
    --quiet \
    --release \
    --target-dir=/tmp/codecrafters-redis-target \
    --manifest-path $(dirname $0)/Cargo.toml "$@"
