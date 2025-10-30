🔨 Building cargo-openvm...
warning: unused import: `unimp`
  --> extensions/rv32im/transpiler/src/lib.rs:14:17
   |
14 |     util::{nop, unimp},
   |                 ^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: `openvm-rv32im-transpiler` (lib) generated 1 warning (run `cargo fix --lib -p openvm-rv32im-transpiler` to apply 1 suggestion)
   Compiling cargo-openvm v1.4.1-rc.1 (/home/cody/openvm/crates/cli)
   Compiling openvm-floats-circuit v1.4.1-rc.1 (/home/cody/openvm/extensions/floats/circuit)
warning: unused import: `openvm_circuit::system::memory::online::GuestMemory`
 --> extensions/floats/circuit/src/handler_executor/operations/fma.rs:2:5
  |
2 | use openvm_circuit::system::memory::online::GuestMemory;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default

warning: unnecessary parentheses around match arm expression
  --> extensions/floats/circuit/src/handler_executor/operations/alu.rs:90:18
   |
90 |             2 => (rs1_val ^ (rs2_val & 0x80000000)),  // FSGNJX: XOR signs
   |                  ^                                ^
   |
   = note: `#[warn(unused_parens)]` on by default
help: remove these parentheses
   |
90 -             2 => (rs1_val ^ (rs2_val & 0x80000000)),  // FSGNJX: XOR signs
90 +             2 => rs1_val ^ (rs2_val & 0x80000000),  // FSGNJX: XOR signs
   |

warning: unused import: `openvm_circuit::system::memory::online::GuestMemory`
 --> extensions/floats/circuit/src/handler_executor/operations/compare.rs:2:5
  |
2 | use openvm_circuit::system::memory::online::GuestMemory;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `openvm_circuit::system::memory::online::GuestMemory`
 --> extensions/floats/circuit/src/handler_executor/operations/class.rs:2:5
  |
2 | use openvm_circuit::system::memory::online::GuestMemory;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `BorrowMut` and `Borrow`
 --> extensions/floats/circuit/src/handler_executor/execution.rs:2:14
  |
2 |     borrow::{Borrow, BorrowMut},
  |              ^^^^^^  ^^^^^^^^^

warning: unused import: `operation::FloatOperation`
 --> extensions/floats/circuit/src/handler_executor/mod.rs:5:9
  |
5 | pub use operation::FloatOperation;
  |         ^^^^^^^^^^^^^^^^^^^^^^^^^

   Compiling openvm-sdk v1.4.1-rc.1 (/home/cody/openvm/crates/sdk)
warning: `openvm-floats-circuit` (lib) generated 6 warnings (run `cargo fix --lib -p openvm-floats-circuit` to apply 6 suggestions)
    Finished `dev` profile [optimized + debuginfo] target(s) in 29.10s
📦 Rebuilding float library...
Building float library for RISCOF tests...
Compiling float.c...
Compiling compiler_builtins.c...
Compiling softfloat_fcsr.c...
Building libziskfloat.a from SoftFloat sources...
Creating static library...
✅ Float library build complete:
-rw-r--r-- 1 cody cody 1.1K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/compiler_builtins.o
-rw-r--r-- 1 cody cody 115K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/float.o
-rw-r--r-- 1 cody cody 134K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/libziskfloat.a
-rw-r--r-- 1 cody cody 1.3K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/softfloat_fcsr.o
📦 Rebuilding tests...
🧪 Testing ZKVMs (arch) (build-only): openvm
🔨 Building RISCOF Docker image...
#0 building with "default" instance using docker driver

#1 [internal] load build definition from Dockerfile
#1 transferring dockerfile: 3.23kB done
#1 DONE 0.0s

#2 [internal] load metadata for docker.io/library/ubuntu:24.04
#2 DONE 0.0s

#3 [internal] load .dockerignore
#3 transferring context: 406B done
#3 DONE 0.0s

#4 [ 1/15] FROM docker.io/library/ubuntu:24.04
#4 DONE 0.0s

#5 [internal] load build context
#5 transferring context: 260.26kB done
#5 DONE 0.0s

#6 [ 3/15] WORKDIR /riscof
#6 CACHED

#7 [13/15] COPY . .
#7 CACHED

#8 [ 2/15] RUN apt-get update && apt-get install -y   python3   python3-pip   python3-venv   curl   git   build-essential   xz-utils   zsh   vim   && rm -rf /var/lib/apt/lists/*
#8 CACHED

#9 [14/15] RUN mkdir -p /dut/plugin /dut/bin /riscof/riscof_work && touch /dut/plugin/dut-exe
#9 CACHED

#10 [ 6/15] RUN curl -L https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2025.08.08/riscv64-elf-ubuntu-24.04-gcc-nightly-2025.08.08-nightly.tar.xz |   tar -xJ -C /riscof/toolchains/
#10 CACHED

#11 [ 9/15] RUN riscof arch-test --clone --get-version 3.9.1
#11 CACHED

#12 [ 4/15] RUN pip3 install --break-system-packages riscof==1.25.3
#12 CACHED

#13 [12/15] RUN PYTHON_SITE_PACKAGES=$(python3 -c "import site; print(site.getsitepackages()[0])") &&   sed -i 's/if key not in list:/if key not in flist:/' "$PYTHON_SITE_PACKAGES/riscof/dbgen.py" &&   sed -i '/def check_commit/,/return str(commit), update/{s/if (str(commit) != old_commit):/update = False\n    if (str(commit) != old_commit):/}' "$PYTHON_SITE_PACKAGES/riscof/dbgen.py"
#13 CACHED

#14 [ 7/15] RUN mv /riscof/toolchains/riscv /riscof/toolchains/riscv64
#14 CACHED

#15 [11/15] RUN patch -d /riscof/riscv-arch-test/ -N -p1 --fuzz=3 < /tmp/patches/fix-c-extension-privilege-tests.patch
#15 CACHED

#16 [ 5/15] RUN mkdir -p toolchains emulators
#16 CACHED

#17 [ 8/15] RUN curl -L https://github.com/riscv/sail-riscv/releases/download/0.7/sail_riscv-Linux-x86_64.tar.gz | tar -xz -C emulators/ &&   mv emulators/sail_riscv-Linux-x86_64 emulators/sail-riscv
#17 CACHED

#18 [10/15] COPY patches/ /tmp/patches/
#18 CACHED

#19 [15/15] RUN echo "{" > /riscof/VERSION.json &&     echo "  "riscof_commit": "unknown"," >> /riscof/VERSION.json &&     echo "  "riscof_version": "1.25.3"," >> /riscof/VERSION.json &&     echo "  "toolchain_version": "2025.08.08"," >> /riscof/VERSION.json &&     echo "  "toolchain_type": "riscv64-elf-ubuntu-24.04"," >> /riscof/VERSION.json &&     echo "  "base_image": "ubuntu:24.04"" >> /riscof/VERSION.json &&     echo "}" >> /riscof/VERSION.json
#19 CACHED

#20 exporting to image
#20 exporting layers done
#20 writing image sha256:08c17e99d08c5dd638ee85ffb523f1544fbbf39496f165e9bf1843a7bad0c64b done
#20 naming to docker.io/library/riscof:latest done
#20 DONE 0.0s
Testing openvm...
Found DUT executable: dut-exe
Found plugin: openvm
Setting up plugin...
📋 Compile-only mode enabled
Generating config.ini for openvm...
Clearing previous results...
Running RISCOF arch tests...
/usr/local/lib/python3.12/dist-packages/riscof/dbgen.py:21: SyntaxWarning: invalid escape sequence '\('
  isa_regex = re.compile('''RVTEST_ISA\(\"(?P<isa>.*)\"\)''')
/usr/local/lib/python3.12/dist-packages/riscof/dbgen.py:22: SyntaxWarning: invalid escape sequence '\('
  case_regex = re.compile('''RVTEST_CASE\((?P<id>.*),\"(?P<cond>.*)\",(?P<cov_label>.*)\)''')
[32m    INFO[0m | [32m****** RISCOF: RISC-V Architectural Test Framework 1.25.3 *******[0m
[32m    INFO[0m | [32musing riscv_isac version : 0.18.0[0m
[32m    INFO[0m | [32musing riscv_config version : 3.18.3[0m
[32m    INFO[0m | [32mReading configuration from: /riscof/config.ini[0m
[32m    INFO[0m | [32mPreparing Models[0m
[32m    INFO[0m | [32mInput-ISA file[0m
[32m    INFO[0m | [32mISACheck: Loading input file: /riscof/plugins/openvm/openvm_isa.yaml[0m
[32m    INFO[0m | [32mISACheck: Load Schema /usr/local/lib/python3.12/dist-packages/riscv_config/schemas/schema_isa.yaml[0m
[32m    INFO[0m | [32mISACheck: Processing Hart:0[0m
[32m    INFO[0m | [32mISACheck: Initiating Validation for Hart:0[0m
[32m    INFO[0m | [32mISACheck: No errors for Hart:0[0m
[32m    INFO[0m | [32mISACheck:  Updating fields node for each CSR in Hart:0[0m
[32m    INFO[0m | [32mISACheck: Dumping out Normalized Checked YAML: /riscof/riscof_work/openvm_isa_checked.yaml[0m
[32m    INFO[0m | [32mInput-Platform file[0m
[32m    INFO[0m | [32mLoading input file: /riscof/plugins/openvm/openvm_platform.yaml[0m
[32m    INFO[0m | [32mLoad Schema /usr/local/lib/python3.12/dist-packages/riscv_config/schemas/schema_platform.yaml[0m
[32m    INFO[0m | [32mInitiating Validation[0m
[32m    INFO[0m | [32mNo Syntax errors in Input Platform Yaml. :)[0m
[32m    INFO[0m | [32mDumping out Normalized Checked YAML: /riscof/riscof_work/openvm_platform_checked.yaml[0m
[32m    INFO[0m | [32mGenerating database for suite: /riscof/riscv-arch-test/riscv-test-suite[0m
[32m    INFO[0m | [32mDatabase File Generated: /riscof/riscof_work/database.yaml[0m
[32m    INFO[0m | [32mEnv path set to/riscof/riscv-arch-test/riscv-test-suite/env[0m
[32m    INFO[0m | [32mRunning Build for DUT[0m
[32m    INFO[0m | [32mFloat support enabled - using locally built library from env/[0m
[32m    INFO[0m | [32m  Library: /riscof/plugins/openvm/env/libziskfloat.a[0m
[32m    INFO[0m | [32mSelecting Tests.[0m
[32m    INFO[0m | [32mRunning Tests on DUT.[0m
[32m    INFO[0m | [32mTests run on DUT done.[0m
  ⚠️  Tests ran but no report generated
✅ Dashboard updated
📦 Rebuilding float library...
Building float library for RISCOF tests...
Compiling float.c...
Compiling compiler_builtins.c...
Compiling softfloat_fcsr.c...
Building libziskfloat.a from SoftFloat sources...
Creating static library...
✅ Float library build complete:
-rw-r--r-- 1 cody cody 1.1K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/compiler_builtins.o
-rw-r--r-- 1 cody cody 115K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/float.o
-rw-r--r-- 1 cody cody 134K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/libziskfloat.a
-rw-r--r-- 1 cody cody 1.3K Oct 30 10:32 /home/cody/openvm/zkevm-test-monitor/riscof/plugins/openvm/env/softfloat_fcsr.o
🧪 Testing ZKVMs (arch): openvm
🔨 Building RISCOF Docker image...
#0 building with "default" instance using docker driver

#1 [internal] load build definition from Dockerfile
#1 transferring dockerfile: 3.23kB done
#1 DONE 0.0s

#2 [internal] load metadata for docker.io/library/ubuntu:24.04
#2 DONE 0.0s

#3 [internal] load .dockerignore
#3 transferring context: 406B done
#3 DONE 0.0s

#4 [ 1/15] FROM docker.io/library/ubuntu:24.04
#4 DONE 0.0s

#5 [internal] load build context
#5 transferring context: 260.26kB done
#5 DONE 0.0s

#6 [ 3/15] WORKDIR /riscof
#6 CACHED

#7 [11/15] RUN patch -d /riscof/riscv-arch-test/ -N -p1 --fuzz=3 < /tmp/patches/fix-c-extension-privilege-tests.patch
#7 CACHED

#8 [ 6/15] RUN curl -L https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2025.08.08/riscv64-elf-ubuntu-24.04-gcc-nightly-2025.08.08-nightly.tar.xz |   tar -xJ -C /riscof/toolchains/
#8 CACHED

#9 [ 7/15] RUN mv /riscof/toolchains/riscv /riscof/toolchains/riscv64
#9 CACHED

#10 [14/15] RUN mkdir -p /dut/plugin /dut/bin /riscof/riscof_work && touch /dut/plugin/dut-exe
#10 CACHED

#11 [ 2/15] RUN apt-get update && apt-get install -y   python3   python3-pip   python3-venv   curl   git   build-essential   xz-utils   zsh   vim   && rm -rf /var/lib/apt/lists/*
#11 CACHED

#12 [12/15] RUN PYTHON_SITE_PACKAGES=$(python3 -c "import site; print(site.getsitepackages()[0])") &&   sed -i 's/if key not in list:/if key not in flist:/' "$PYTHON_SITE_PACKAGES/riscof/dbgen.py" &&   sed -i '/def check_commit/,/return str(commit), update/{s/if (str(commit) != old_commit):/update = False\n    if (str(commit) != old_commit):/}' "$PYTHON_SITE_PACKAGES/riscof/dbgen.py"
#12 CACHED

#13 [13/15] COPY . .
#13 CACHED

#14 [ 8/15] RUN curl -L https://github.com/riscv/sail-riscv/releases/download/0.7/sail_riscv-Linux-x86_64.tar.gz | tar -xz -C emulators/ &&   mv emulators/sail_riscv-Linux-x86_64 emulators/sail-riscv
#14 CACHED

#15 [ 4/15] RUN pip3 install --break-system-packages riscof==1.25.3
#15 CACHED

#16 [10/15] COPY patches/ /tmp/patches/
#16 CACHED

#17 [ 9/15] RUN riscof arch-test --clone --get-version 3.9.1
#17 CACHED

#18 [ 5/15] RUN mkdir -p toolchains emulators
#18 CACHED

#19 [15/15] RUN echo "{" > /riscof/VERSION.json &&     echo "  "riscof_commit": "unknown"," >> /riscof/VERSION.json &&     echo "  "riscof_version": "1.25.3"," >> /riscof/VERSION.json &&     echo "  "toolchain_version": "2025.08.08"," >> /riscof/VERSION.json &&     echo "  "toolchain_type": "riscv64-elf-ubuntu-24.04"," >> /riscof/VERSION.json &&     echo "  "base_image": "ubuntu:24.04"" >> /riscof/VERSION.json &&     echo "}" >> /riscof/VERSION.json
#19 CACHED

#20 exporting to image
#20 exporting layers done
#20 writing image sha256:08c17e99d08c5dd638ee85ffb523f1544fbbf39496f165e9bf1843a7bad0c64b done
#20 naming to docker.io/library/riscof:latest done
#20 DONE 0.0s
Testing openvm...
Found DUT executable: dut-exe
Found plugin: openvm
Setting up plugin...
Generating config.ini for openvm...
Clearing previous results...
Running RISCOF arch tests...
/usr/local/lib/python3.12/dist-packages/riscof/dbgen.py:21: SyntaxWarning: invalid escape sequence '\('
  isa_regex = re.compile('''RVTEST_ISA\(\"(?P<isa>.*)\"\)''')
/usr/local/lib/python3.12/dist-packages/riscof/dbgen.py:22: SyntaxWarning: invalid escape sequence '\('
  case_regex = re.compile('''RVTEST_CASE\((?P<id>.*),\"(?P<cond>.*)\",(?P<cov_label>.*)\)''')
[32m    INFO[0m | [32m****** RISCOF: RISC-V Architectural Test Framework 1.25.3 *******[0m
[32m    INFO[0m | [32musing riscv_isac version : 0.18.0[0m
[32m    INFO[0m | [32musing riscv_config version : 3.18.3[0m
[32m    INFO[0m | [32mReading configuration from: /riscof/config.ini[0m
[32m    INFO[0m | [32mPreparing Models[0m
[32m    INFO[0m | [32mInput-ISA file[0m
[32m    INFO[0m | [32mISACheck: Loading input file: /riscof/plugins/openvm/openvm_isa.yaml[0m
[32m    INFO[0m | [32mISACheck: Load Schema /usr/local/lib/python3.12/dist-packages/riscv_config/schemas/schema_isa.yaml[0m
[32m    INFO[0m | [32mISACheck: Processing Hart:0[0m
[32m    INFO[0m | [32mISACheck: Initiating Validation for Hart:0[0m
[32m    INFO[0m | [32mISACheck: No errors for Hart:0[0m
[32m    INFO[0m | [32mISACheck:  Updating fields node for each CSR in Hart:0[0m
[32m    INFO[0m | [32mISACheck: Dumping out Normalized Checked YAML: /riscof/riscof_work/openvm_isa_checked.yaml[0m
[32m    INFO[0m | [32mInput-Platform file[0m
[32m    INFO[0m | [32mLoading input file: /riscof/plugins/openvm/openvm_platform.yaml[0m
[32m    INFO[0m | [32mLoad Schema /usr/local/lib/python3.12/dist-packages/riscv_config/schemas/schema_platform.yaml[0m
[32m    INFO[0m | [32mInitiating Validation[0m
[32m    INFO[0m | [32mNo Syntax errors in Input Platform Yaml. :)[0m
[32m    INFO[0m | [32mDumping out Normalized Checked YAML: /riscof/riscof_work/openvm_platform_checked.yaml[0m
[32m    INFO[0m | [32mGenerating database for suite: /riscof/riscv-arch-test/riscv-test-suite[0m
[32m    INFO[0m | [32mDatabase File Generated: /riscof/riscof_work/database.yaml[0m
[32m    INFO[0m | [32mEnv path set to/riscof/riscv-arch-test/riscv-test-suite/env[0m
[32m    INFO[0m | [32mRunning Build for DUT[0m
[32m    INFO[0m | [32mFloat support enabled - using locally built library from env/[0m
[32m    INFO[0m | [32m  Library: /riscof/plugins/openvm/env/libziskfloat.a[0m
[32m    INFO[0m | [32mRunning Build for Reference[0m
[32m    INFO[0m | [32mSelecting Tests.[0m
[32m    INFO[0m | [32mRunning Tests on DUT.[0m
[32m    INFO[0m | [32mRunning Tests on Reference Model.[0m
[32m    INFO[0m | [32mInitiating signature checking.[0m
[32m    INFO[0m | [32mFollowing 396 tests have been run :
[0m
[32m    INFO[0m | [32mTEST NAME                                          : COMMIT ID                                : STATUS[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b10-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b11-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b12-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b13-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fadd_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fclass_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.s.w_b25-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.s.w_b26-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.s.wu_b25-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.s.wu_b26-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b22-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b23-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b24-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b27-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b28-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.w.s_b29-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b22-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b23-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b24-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b27-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b28-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fcvt.wu.s_b29-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b20-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b21-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b6-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fdiv_b9-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/feq_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/feq_b19-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fle_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fle_b19-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/flt_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/flt_b19-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/flw-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b14-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-001.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-002.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-003.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-004.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-005.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-006.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-007.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-008.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-009.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-010.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-011.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-012.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-013.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-014.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-015.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-016.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-017.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-018.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-019.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-020.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-021.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-022.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-023.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-024.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-025.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-026.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-027.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-028.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-029.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-030.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-031.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-032.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-033.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-034.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-035.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-036.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-037.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-038.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-039.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-040.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-041.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-042.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-043.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-044.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-045.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-046.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-047.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-048.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-049.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b15/fmadd_b15-050.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b16-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b17-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b18-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b6-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmadd_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmax_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmax_b19-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmin_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmin_b19-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b14-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-001.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-002.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-003.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-004.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-005.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-006.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-007.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-008.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-009.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-010.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-011.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-012.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-013.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-014.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-015.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-016.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-017.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-018.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-019.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-020.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-021.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-022.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-023.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-024.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-025.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-026.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-027.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-028.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-029.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-030.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-031.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-032.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-033.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-034.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-035.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-036.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-037.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-038.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-039.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-040.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-041.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-042.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-043.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-044.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-045.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-046.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-047.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-048.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-049.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b15/fmsub_b15-050.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b16-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b17-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b18-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b6-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmsub_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b6-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmul_b9-01.S : c61488a21e631bad51116b48f1f9d63232e81ed3 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.w.x_b25-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.w.x_b26-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b22-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b23-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b24-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b27-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b28-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fmv.x.w_b29-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Failed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b14-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-001.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-002.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-003.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-004.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-005.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-006.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-007.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-008.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-009.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-010.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-011.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-012.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-013.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-014.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-015.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-016.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-017.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-018.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-019.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-020.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-021.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-022.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-023.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-024.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-025.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-026.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-027.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-028.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-029.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-030.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-031.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-032.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-033.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-034.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-035.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-036.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-037.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-038.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-039.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-040.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-041.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-042.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-043.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-044.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-045.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-046.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-047.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-048.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-049.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b15/fnmadd_b15-050.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b16-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b17-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b18-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b6-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmadd_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b14-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-001.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-002.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-003.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-004.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-005.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-006.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-007.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-008.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-009.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-010.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-011.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-012.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-013.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-014.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-015.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-016.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-017.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-018.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-019.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-020.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-021.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-022.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-023.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-024.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-025.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-026.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-027.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-028.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-029.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-030.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-031.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-032.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-033.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-034.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-035.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-036.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-037.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-038.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-039.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-040.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-041.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-042.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-043.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-044.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-045.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-046.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-047.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-048.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-049.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b15/fnmsub_b15-050.S : 0d6fa5ca1dceaa916180d7d800781ca67a554931 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b16-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b17-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b18-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b6-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fnmsub_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsgnj_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsgnjn_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsgnjx_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b20-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsqrt_b9-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b1-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b10-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b11-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b12-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b13-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b2-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b3-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b4-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b5-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b7-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsub_b8-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/F/src/fsw-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/add-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/addi-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/and-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/andi-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/auipc-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/beq-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/bge-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/bgeu-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/blt-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/bltu-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/bne-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/fence-01.S : 81c7a2b769baa2f33f40bc5455299b1362b5d125 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/jal-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/jalr-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/lb-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/lbu-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/lh-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/lhu-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/lui-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/lw-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/misalign1-jalr-01.S : 0c4cdffe19b1a48d9fec8590c8817af2ff924a37 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/or-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/ori-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sb-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sh-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sll-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/slli-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/slt-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/slti-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sltiu-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sltu-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sra-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/srai-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/srl-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/srli-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sub-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/sw-align-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/xor-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/I/src/xori-01.S : b91f98f3a0e908bad4680c2e3901fbc24b63a563 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/div-01.S : 9b503d7890296e53aa8a06e49ebef3c61ce5d3fd : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/divu-01.S : 3c7e9d41d4efb9dcb9c0af83e0eecbe28327bf3c : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/mul-01.S : 3c7e9d41d4efb9dcb9c0af83e0eecbe28327bf3c : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/mulh-01.S : 3c7e9d41d4efb9dcb9c0af83e0eecbe28327bf3c : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/mulhsu-01.S : a02feaee118fbea01fbb8fdcdf62bce6f7067478 : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/mulhu-01.S : 3c7e9d41d4efb9dcb9c0af83e0eecbe28327bf3c : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/rem-01.S : 3c7e9d41d4efb9dcb9c0af83e0eecbe28327bf3c : Passed[0m
[32m    INFO[0m | [32m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/M/src/remu-01.S : 3c7e9d41d4efb9dcb9c0af83e0eecbe28327bf3c : Passed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/ebreak.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/ecall.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/misalign-lh-01.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/misalign-lhu-01.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/misalign-lw-01.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/misalign-sh-01.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[1;31m   ERROR[0m | [1;31m/riscof/riscv-arch-test/riscv-test-suite/rv32i_m/privilege/src/misalign-sw-01.S : bb74a4aefaa8c89fb28b876484cfdf9ca020cec1 : Failed[0m
[32m    INFO[0m | [32mTest report generated at /riscof/riscof_work/report.html.[0m
[32m    INFO[0m | [32mOpening test report in web-browser[0m
  ✅ Tested openvm: 388/396 passed
✅ Dashboard updated
