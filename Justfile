_default:
    @just --list

doc:
    cargo make doc

clippy:
    cargo make clippy

qemu:
    cargo make qemu

build:
    cargo make kernel-bin

gdb:
    cargo make qemu-gdb
