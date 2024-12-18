#!/bin/bash
cargo build --target aarch64-unknown-linux-gnu
sshpass -p "hypefuse" scp target/aarch64-unknown-linux-gnu/debug/nemoscene hypefuse@linfinitysmartmirror.local:/hypefuse
