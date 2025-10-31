We are implementing an extension of OpenVM, which handles rv32im execution, so that we can handle rv32imf execution. We are doing this by software emulation using lib-float, written by ZisK.

We are very confident that the existing rv32im system works, and we will not make any fundamental logical changes to the transpiler or the interpreter for that instruction set. We also know with certainty that lib-float is a correct implementation of RV64IMAFDC, since we have run the compliance tests for this architecture many times (the only tests that fail are some of the Zicsr tests, which is expected since ZisK does not fully support the privileged architecture).

We have the RISC-V architecture tests set up to run in zkevm-test-monitor, which is a symlink to an external repo. Rather than interact with these directly, you will the scripts arch-one.sh and arch-all.sh. Familiarize yourself with those scripts. Only after arch-one.sh works for a one or mo will you try to run the full suite with arch-all.sh. In the case that we want to make changes to a test file, such as we did with fnmadd_b1 (we wanted to cut out many tests since it was so huge) we have the extra test suite which can be run with extra-debug.sh.

A RISC-V set of GNU binutils is available at riscv64-elf-* (note that the word "unknown" does not appear).
