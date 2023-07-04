PROFILE?=debug
TARGET?=aarch64-unknown-none
O=target/$(TARGET)/$(PROFILE)

CARGO_BUILD_ARGS=-Z build-std=core \
				 --target etc/$(TARGET).json
ifeq ($(PROFILE),release)
CARGO_BUILD_ARGS+=--release
endif

QEMU_OPTS=-s \
		  -serial mon:stdio \
		  -cpu cortex-a57 \
		  -M virt \
		  -display none

ifeq ($(QEMU_PAUSE),1)
QEMU_OPTS+=-S
endif
ifeq ($(QEMU_DINT),1)
QEMU_OPTS+=-d int
endif

all: kernel

kernel-raw:
	cargo build $(CARGO_BUILD_ARGS)

kernel: kernel-raw
	cp $(O)/osdev-x $(O)/kernel

kernel-bin: kernel
	llvm-objcopy -O binary $(O)/kernel $(O)/kernel.bin

qemu: kernel-bin
	qemu-system-aarch64 $(QEMU_OPTS) -kernel $(O)/kernel.bin
