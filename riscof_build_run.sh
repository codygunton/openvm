#!/bin/bash
set -eu

# Rebuild flags
BTESTS=${BTESTS:-0}
BBIN=${BBIN:-0}
RUN=${RUN:-1}
SIGS=${SIGS:-0}

PATTERN=${1:-add}

BINARY_TO_RUN=zkevm-test-monitor/binaries/openvm-binary
rm -f $BINARY_TO_RUN

# Build cargo-openvm from the openvm repository
if [ $BBIN = "1" ]; then
    echo "🔨 Building cargo-openvm..."
    cargo build -p cargo-openvm
fi

# Copy the binary to zkevm-test-monitor
cp target/debug/cargo-openvm $BINARY_TO_RUN

# Run the test
cd zkevm-test-monitor

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

# Optionally rebuild tests (set BTESTS=1 to enable)
if [ $BTESTS = "1" ]; then
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

        # Generate objdump if ELF exists
        if [ -f "$ELF_FILE" ]; then
            echo "📝 Generating objdump..."

            # Create dump in a temp location first (in case test dir is read-only)
            TEMP_DUMP="/tmp/riscof-test.dump"
            riscv64-elf-objdump -S "$ELF_FILE" > "$TEMP_DUMP"

            # Create symlinks in repo base directory
            cd ..
            rm -f riscof-test.elf riscof-test.dump

            ln -s "zkevm-test-monitor/$ELF_FILE" riscof-test.elf
            ln -s "$TEMP_DUMP" riscof-test.dump

            cd zkevm-test-monitor
        fi
    else
        echo "⚠️  Could not find test directory for pattern: $PATTERN"
    fi

    # Compare signatures if SIGS=1
    if [ $SIGS = "1" ] && [ -n "$TEST_DIR" ] && [ -d "$TEST_DIR" ]; then
        echo "🔍 Comparing signatures..."

        # Get DUT signature from test results directory
        DUT_SIG="$TEST_DIR/DUT-openvm.signature"

        # Look for reference signature in base directory
        cd ..
        REF_SIG="Reference-sail_c_simulator.signature"

        if [ ! -f "zkevm-test-monitor/$DUT_SIG" ]; then
            echo "   ⚠️  DUT signature not found: zkevm-test-monitor/$DUT_SIG"
            echo "   The test may not have generated a signature file"
            cd zkevm-test-monitor
        elif [ ! -f "$REF_SIG" ]; then
            echo "   ⚠️  Reference signature not found: $REF_SIG"
            echo "   Copy it from test results first:"
            echo "   cp zkevm-test-monitor/test-results/openvm/.../ref/Reference-sail_c_simulator.signature ."
            cd zkevm-test-monitor
        else
            echo "   ✓ DUT signature: zkevm-test-monitor/$DUT_SIG"
            echo "   ✓ REF signature: $REF_SIG"

            # Compare the signatures
            DIFF_OUTPUT=$(mktemp)
            diff -y --suppress-common-lines "$REF_SIG" "zkevm-test-monitor/$DUT_SIG" > "$DIFF_OUTPUT" 2>&1 || true

            if [ -s "$DIFF_OUTPUT" ]; then
                MISMATCH_COUNT=$(wc -l < "$DIFF_OUTPUT")
                echo "❌ Found $MISMATCH_COUNT mismatched lines (showing first 10):"

                # Create formatted comparison output
                {
                    echo "   Line | DUT (actual)   | REF (expected)"
                    echo "   ─────┼────────────────┼────────────────"
                    head -n 10 "$DIFF_OUTPUT" | awk '{
                        # Extract left (REF) and right (DUT) values
                        split($0, parts, /[<>|]/)
                        ref = parts[1]
                        dut = parts[length(parts)]
                        gsub(/^[ \t]+|[ \t]+$/, "", ref)
                        gsub(/^[ \t]+|[ \t]+$/, "", dut)
                        printf "   %4d │ %-14s │ %s\n", NR, dut, ref
                    }'
                }

                # Create symlink to DUT signature for easy access
                rm -f DUT-openvm.signature
                ln -s "zkevm-test-monitor/$DUT_SIG" DUT-openvm.signature
                echo "   📝 Full signatures available:"
                echo "      DUT-openvm.signature (newly generated)"
                echo "      Reference-sail_c_simulator.signature (reference)"
            else
                echo "✅ Signatures match perfectly!"
            fi

            rm -f "$DIFF_OUTPUT"
            cd zkevm-test-monitor
        fi
    fi

    if [ $EXIT_CODE -ne 0 ]; then
        echo "⚠️  Test exited with code: $EXIT_CODE"
    else
        echo "✅ Test passed"
    fi

    exit $EXIT_CODE
fi
