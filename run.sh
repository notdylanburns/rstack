#!/bin/bash

set -e

cargo build --release
sudo setcap cap_net_admin=eip target/release/rstack

target/release/rstack &
pid=$!

trap "kill $pid" EXIT

sudo ip addr add '192.168.0.1' dev tap0
sudo ip link set dev tap0 up

wait $pid