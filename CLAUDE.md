We are implementing an extension of OpenVM, which handles rv32im execution, so that we can handle rv32imf execution. 

We are very confident that the existing rv32im system works, and we will not make any fundamental logical changes to the transpiler or the interpreter.

We have the RISC-V architecture tests set up to run in zkevm-test-monitor, which is a symlink to an external repo. Rather than interact with these directly, you will the scripts arch-one.sh and arch-all.sh. Familiarize yourself with those scripts. Only after arch-one.sh works for a few different instructions will you try to run the full suite with arch-all.sh.

A RISC-V set of GNU binutils is available at riscv64-elf-* (note that the word "unknown" does not appear).
