#!/bin/bash
export PYO3_CROSS_PYTHON_VERSION=3.12
export PKG_CONFIG_PATH=${PWD}/libpython
echo $PKG_CONFIG_PATH
cargo build --target aarch64-unknown-linux-gnu
sshpass -p "hypefuse" scp target/aarch64-unknown-linux-gnu/debug/nemoscene hypefuse@linfinitysmartmirror.local:/hypefuse
