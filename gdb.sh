#!/bin/sh

if [ "${PROFILE}" = "" ]; then
    PROFILE=debug
fi
if [ "${TARGET}" = "" ]; then
    TARGET=aarch64-unknown-none
fi

aarch64-linux-gnu-gdb target/${TARGET}/${PROFILE}/osdev-x
