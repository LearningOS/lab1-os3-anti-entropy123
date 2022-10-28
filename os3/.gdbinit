# target remote localhost:1234
file target/riscv64gc-unknown-none-elf/release/os
focus cmd
dir src
set confirm off
set architecture riscv:rv64
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes
