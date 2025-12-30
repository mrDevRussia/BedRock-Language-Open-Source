#!/bin/bash
set -e

# Usage: ./build_iso.sh <input_object_file> <output_iso_name>

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <input_object_file> <output_iso_name>"
    exit 1
fi

echo "[BedRock] Linking kernel..."
ld -m elf_x86_64 -T toolchain/linker.ld -o kernel.elf $1 --oformat elf64-x86-64

echo "[BedRock] Creating ISO directory structure..."
mkdir -p isodir/boot/grub

cp kernel.elf isodir/boot/kernel.bin

# Create minimal grub.cfg
cat > isodir/boot/grub/grub.cfg << EOF
set timeout=0
set default=0

menuentry "BedRock OS" {
    multiboot /boot/kernel.bin
    boot
}
EOF

echo "[BedRock] Generating ISO image..."
# Requires grub-mkrescue (usually part of grub2-common/xorriso)
grub-mkrescue -o $2 isodir

echo "[BedRock] Cleaning up..."
rm -rf isodir kernel.elf

echo "[BedRock] Build Complete: $2"
