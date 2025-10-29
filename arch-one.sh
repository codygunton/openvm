#!/bin/bash
set -eu

# Rebuild flags
BTESTS=${BTESTS:-0}
BBIN=${BBIN:-0}
RUN=${RUN:-1}
SIGS=${SIGS:-1}

PATTERN=${1:-fadd_b1}

BINARY_TO_RUN=zkevm-test-monitor/binaries/openvm-binary

# Build cargo-openvm from the openvm repository
if [ $BBIN = "1" ]; then
    rm -f $BINARY_TO_RUN
    echo "🔨 Building cargo-openvm..."
    cargo build -p cargo-openvm
    # Copy the binary to zkevm-test-monitor
    cp target/debug/cargo-openvm $BINARY_TO_RUN
fi

# Run the test
cd zkevm-test-monitor

# Optionally rebuild tests (set BTESTS=1 to enable)
if [ $BTESTS = "1" ]; then
    # Build float library if needed
    # Prefer using artifacts from cargo-openvm build (single source of truth)
    if [ -f "binaries/float-libs/libziskfloat.a" ]; then
        echo "🔧 Using float library from cargo-openvm build..."
        cp binaries/float-libs/libziskfloat.a riscof/plugins/openvm/env/
        # Also compile the small local files (float.o, compiler_builtins.o)
        # These are quick to compile and need to match the exact compiler flags
        if [ ! -f "riscof/plugins/openvm/env/float.o" ]; then
            riscof/plugins/openvm/env/build_float_lib.sh
        fi
    elif [ ! -f "riscof/plugins/openvm/env/libziskfloat.a" ]; then
        echo "🔧 Building float library locally..."
        riscof/plugins/openvm/env/build_float_lib.sh
    fi

    echo "📦 Rebuilding float library..."
    riscof/plugins/openvm/env/build_float_lib.sh
    echo "📦 Rebuilding tests..."
    ./run test --arch openvm --build-only
else
    echo "⏭️  Skipping test rebuild (set BTESTS=1 to rebuild)"
fi

# Run the specific test in debug mode
if [ $RUN = "1" ]; then
    echo "🐛 Running debug test: $PATTERN"

    # Run test and capture exit code, showing only last 20 lines
    set +e # Temporarily disable exit on error
    ./run debug --arch openvm "$PATTERN" 2>&1 | tail -20
    EXIT_CODE=${PIPESTATUS[0]}
    set -e # Re-enable exit on error

    # Find the test directory and files
    # First try exact match (e.g., fadd_b1 matches fadd_b1-01.S but not fadd_b10-01.S)
    TEST_DIR=$(find test-results/openvm -type d -path "*/${PATTERN}-*" -path "*/dut" | head -1)
    # Fall back to substring match if exact fails
    if [ -z "$TEST_DIR" ]; then
        TEST_DIR=$(find test-results/openvm -type d -path "*${PATTERN}*" -path "*/dut" | head -1)
    fi

    if [ -n "$TEST_DIR" ] && [ -d "$TEST_DIR" ]; then
        ELF_FILE="$TEST_DIR/my.elf"
        echo "ELF_FILES IS $ELF_FILE"

        # Generate objdump if ELF exists
        if [ -f "$ELF_FILE" ]; then
            echo "📝 Generating objdump..."

            cd ..
            rm -rf arch-test.elf arch-test.dump arch-test.log
            TEST_DUMP="arch-test.dump"
            DUT_LOG="zkevm-test-monitor/debug-output/openvm/temp_work/latest.log"
            ln -s $DUT_LOG arch-test.log

            riscv64-elf-objdump -S "zkevm-test-monitor/$ELF_FILE" > "$TEST_DUMP"
            ln -s "zkevm-test-monitor/$ELF_FILE" arch-test.elf

            cd zkevm-test-monitor
        fi
    else
        echo "⚠️  Could not find test directory for pattern: $PATTERN"
    fi

    # Compare signatures if SIGS=1
    if [ $SIGS = "1" ] && [ -n "$TEST_DIR" ] && [ -d "$TEST_DIR" ]; then
        echo "🔍 Comparing signatures..."

        # Get DUT signature from debug output (more up-to-date than test results)
        DUT_SIG="debug-output/openvm/debug.signature"

        # Derive reference signature path from test directory
        # TEST_DIR is like: test-results/openvm/rv32i_m/F/src/fadd_b1-01.S/dut
        # We want: ref-sigs/F/src/fadd_b1-01.S/ref/Reference-sail_c_simulator.signature
        TEST_PATH=$(echo "$TEST_DIR" | sed 's|test-results/openvm/rv32i_m/||' | sed 's|/dut$||')
        cd ..
        REF_SIG="ref-sigs/${TEST_PATH}/ref/Reference-sail_c_simulator.signature"

        if [ ! -f "zkevm-test-monitor/$DUT_SIG" ]; then
            echo "   ⚠️  DUT signature not found: zkevm-test-monitor/$DUT_SIG"
            echo "   The test may not have generated a signature file"
            cd zkevm-test-monitor
        elif [ ! -f "$REF_SIG" ]; then
            echo "   ⚠️  Reference signature not found: $REF_SIG"
            echo "   The reference signature should be in ref-sigs following the test directory structure"
            cd zkevm-test-monitor
        else
            echo "   ✓ DUT signature: zkevm-test-monitor/$DUT_SIG"
            echo "   ✓ REF signature: $REF_SIG"

            # Compare the signatures using delta
            echo "❌ Signature differences; first 40 lines:"
            delta -w 48 "zkevm-test-monitor/$DUT_SIG" $REF_SIG | head -40

            # Count total differences
            DIFF_COUNT=$(diff "$REF_SIG" "zkevm-test-monitor/$DUT_SIG" | wc -l || true)
            if [ "$DIFF_COUNT" -gt 0 ]; then
                echo ""
                echo "   Total diff lines: $DIFF_COUNT"

                # Create symlink to DUT signature for easy access
                rm -f DUT-openvm.signature
                ln -s "zkevm-test-monitor/$DUT_SIG" DUT-openvm.signature
                echo "   📝 Full signatures available:"
                echo "      DUT-openvm.signature (newly generated)"
                echo "      Reference-sail_c_simulator.signature (reference)"
            else
                echo "✅ Signatures match perfectly!"
            fi

            cd zkevm-test-monitor
        fi
    fi

    if [ $EXIT_CODE -ne 0 ]; then
        echo "⚠️  Test exited with code: $EXIT_CODE"
    fi

    exit $EXIT_CODE
fi
