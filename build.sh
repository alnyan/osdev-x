#!/bin/sh

set -e

WORKSPACE_ROOT=$(pwd)
ARCH=aarch64

if [ "${PROFILE}" = "" ]; then
    PROFILE=debug
fi

QEMU=qemu-system-aarch64

KERNEL_TARGET=${ARCH}-unknown-none
USER_TARGET=${ARCH}-unknown-yggdrasil

QEMU_OPTS=" \
    -s \
    -M virt \
    -cpu cortex-a57 \
    -smp 4 \
    -serial mon:stdio \
    -display none \
"

KERNEL_CARGO_OPTS=" \
    --target=${WORKSPACE_ROOT}/etc/${KERNEL_TARGET}.json \
    -Z build-std=core,alloc,compiler_builtins"

USER_CARGO_OPTS="--target=${USER_TARGET}"

if [ "${PROFILE}" = release ]; then
    KERNEL_CARGO_OPTS="${KERNEL_CARGO_OPTS} --release"
    USER_CARGO_OPTS="${USER_CARGO_OPTS} --release"
fi

KERNEL_OUTPUT_DIR=target/${KERNEL_TARGET}/${PROFILE}

pstatus() {
    echo -e "[BUILD] \033[32;1m$@\033[0m"
}

run_cargo() {
    local crate_dir="${1}"
    shift
    local cargo_opts="${@}"

    cd "${crate_dir}"
    cargo ${cargo_toolchain} ${cargo_opts}
    cd -
}

build_kernel() {
    pstatus "Build kernel"
    PROFILE=${PROFILE} run_cargo kernel build ${KERNEL_CARGO_OPTS}
}

build_kernel_bin() {
    pstatus "Creating kernel binary"
    llvm-objcopy -O binary ${KERNEL_OUTPUT_DIR}/kernel ${KERNEL_OUTPUT_DIR}/kernel.bin
}

build_user_program() {
    pstatus "Build user program \"${1}\""
    PROFILE=${PROFILE} cargo_toolchain=+ygg-stage1 run_cargo usr/${1} build ${USER_CARGO_OPTS}
}

build_test_program() {
    build_user_program "test_program"
}

build() {
    build_test_program
    build_kernel
    build_kernel_bin
}

case "$1" in
    build | "")
        build
        ;;
    check | clippy)
        PROFILE=${PROFILE} cargo_toolchain=+ygg-stage1 run_cargo usr/test_program $1 ${USER_CARGO_OPTS}
        PROFILE=${PROFILE} run_cargo kernel $1 ${KERNEL_CARGO_OPTS}
        ;;
    test)
        PROFILE=${PROFILE} run_cargo lib/vfs $1
        ;;
    qemu)
        build
        shift
        "${QEMU}" -kernel ${KERNEL_OUTPUT_DIR}/kernel.bin  ${QEMU_OPTS} $@
        ;;
    *)
        ;;
esac
