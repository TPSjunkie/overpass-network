#!/bin/bash

# Build the project
cargo build --release

# Copy the binary to the target directory
cp target/release/overpass-channels ./target/release/overpass-channels.so

# Copy the binary to the scripts directory
cp target/release/overpass-channels ./scripts/overpass-channels.so
