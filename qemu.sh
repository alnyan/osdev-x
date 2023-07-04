#!/bin/sh
set -e

PROFILE=debug
TARGET=aarch64-unknown-none
O=target/${TARGET}/${PROFILE}

cargo build --target=etc/${TARGET}.json -Z build-std=core
llvm-objcopy -O binary ${O}/osdev-x ${O}/osdev-x.bin

QEMU_OPTS=

if [ "${QEMU_PAUSE}" = 1 ]; then
    QEMU_OPTS="${QEMU_OPTS} -S"
fi

qemu-system-aarch64 \
    -cpu cortex-a76 \
    -M virt \
    -kernel ${O}/osdev-x.bin \
    -serial mon:stdio \
    -display none \
    -d int \
    -s ${QEMU_OPTS}
