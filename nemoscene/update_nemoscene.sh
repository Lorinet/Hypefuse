#!/bin/bash
cargo build --target arm-unknown-linux-gnueabihf
sshpass -p "hypefuse" scp target/arm-unknown-linux-gnueabihf/debug/nemoscene hypefuse@linfinitysmartmirror.local:/hypefuse
