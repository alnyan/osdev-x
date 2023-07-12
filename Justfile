PROFILE := env_var_or_default('PROFILE', 'development')

_default:
    @just --list

doc:
    cargo make doc

clippy:
    cargo make clippy

check:
    cargo make check

qemu:
    cargo make --profile={{PROFILE}} qemu

build:
    cargo make --profile={{PROFILE}} kernel-bin

gdb:
    cargo make --profile={{PROFILE}} qemu-gdb
