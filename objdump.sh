#!/bin/bash

# Check if an argument was provided
if [ -z "$1" ]; then
    echo "Error: Missing memory address argument."
    echo "Usage: $0 <hex_address>"
    exit 1
fi

# Store the first argument into a descriptive variable
TARGET_ADDR="$1"

# Define the binary path relative to where the script is run
BINARY_PATH="target/riscv64gc-unknown-none-elf/debug/OS"

# Check if the compiled binary actually exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Please run 'cargo build' first."
    exit 1
fi

# Execute the disassembly pipeline
echo "Searching for address '$TARGET_ADDR' in $BINARY_PATH..."
riscv64-unknown-elf-objdump -d "$BINARY_PATH" | grep -i -C 15 "$TARGET_ADDR"

echo "Finding Code corresponding to the target address: '$TARGET_ADDR' ..."
addr2line -e target/riscv64gc-unknown-none-elf/debug/OS -f -C "$TARGET_ADDR"