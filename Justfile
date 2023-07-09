_default:
    @just --list

qemu:
    cargo make qemu

build:
    cargo make kernel-bin

gdb:
    cargo make qemu-gdb
