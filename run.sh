#!/bin/bash

set -e

cargo build --release
sudo setcap cap_net_admin=eip target/release/rstack

target/release/rstack &
pid=$!

trap "kill $pid" EXIT

sudo ip link set dev tap0 up

wait $pid