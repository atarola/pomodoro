#!/bin/sh
cargo objcopy --release -- -O binary ./target/pomodoro.bin
# N.B. 0x26000 is for the S140 v6, use 0x27000 for the S140 v7
uf2conv ./target/pomodoro.bin -f 0xADA52840 -b 0x26000 -o ./target/pomodoro.uf2
