#!/bin/bash

set -e  # stop if any command fails

echo "🔨 Building OS..."
cargo build

echo "🚀 Running in QEMU..."
qemu-system-riscv64 \
  -machine virt \
  -nographic \
  -serial mon:stdio \
  -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/OS