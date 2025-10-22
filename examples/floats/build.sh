#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FLOAT_LIB_DIR="$SCRIPT_DIR/../../extensions/floats/guest/vendor/zisk/lib-float/c"
SOFTFLOAT_SRC="$FLOAT_LIB_DIR/SoftFloat-3e/source"
SOFTFLOAT_8086="$SOFTFLOAT_SRC/8086"
SOFTFLOAT_BUILD="$FLOAT_LIB_DIR/SoftFloat-3e/build/Linux-x86_64-GCC"

# Output directory
OUT_DIR="$SCRIPT_DIR/build"
mkdir -p "$OUT_DIR"

echo "Building minimal float test with SoftFloat library..."

# Compiler flags for rv32imf
CFLAGS="-march=rv32imf -mabi=ilp32f -ffreestanding -nostdlib -mcmodel=medany -O3 -DZISK_GCC"
CFLAGS="$CFLAGS -I$SOFTFLOAT_SRC/include -I$SOFTFLOAT_BUILD -I$SOFTFLOAT_8086 -I$FLOAT_LIB_DIR/src/float"

# Compile the assembly test file
echo "Compiling test.S..."
riscv64-elf-gcc $CFLAGS -c "$SCRIPT_DIR/test.S" -o "$OUT_DIR/test.o"

# Compile the float library
echo "Compiling float.c..."
riscv64-elf-gcc $CFLAGS -c "$FLOAT_LIB_DIR/src/float/float.c" -o "$OUT_DIR/float.o"

# Compile __ucmpdi2 implementation for rv32 (needed - libgcc doesn't provide it)
# Use the shared implementation from the guest vendor directory
echo "Compiling compiler_builtins.c (provides __ucmpdi2)..."
COMPILER_BUILTINS="$SCRIPT_DIR/../../extensions/floats/guest/vendor/compiler_builtins.c"
riscv64-elf-gcc $CFLAGS -c "$COMPILER_BUILTINS" -o "$OUT_DIR/compiler_builtins.o"

# Compile essential SoftFloat files
echo "Compiling SoftFloat files..."
compile_softfloat() {
    local src="$1"
    local out="$2"
    riscv64-elf-gcc $CFLAGS -c "$SOFTFLOAT_SRC/$src" -o "$OUT_DIR/$out"
}

compile_softfloat_8086() {
    local src="$1"
    local out="$2"
    riscv64-elf-gcc $CFLAGS -c "$SOFTFLOAT_8086/$src" -o "$OUT_DIR/$out"
}

# Core float operations
compile_softfloat "f32_add.c" "f32_add.o"
compile_softfloat "f64_add.c" "f64_add.o"
compile_softfloat "f32_sub.c" "f32_sub.o"
compile_softfloat "f64_sub.c" "f64_sub.o"
compile_softfloat "f32_mul.c" "f32_mul.o"
compile_softfloat "f64_mul.c" "f64_mul.o"
compile_softfloat "f32_div.c" "f32_div.o"
compile_softfloat "f64_div.c" "f64_div.o"
compile_softfloat "f32_sqrt.c" "f32_sqrt.o"
compile_softfloat "f64_sqrt.c" "f64_sqrt.o"
compile_softfloat "f32_mulAdd.c" "f32_mulAdd.o"
compile_softfloat "f64_mulAdd.c" "f64_mulAdd.o"

# Comparisons
compile_softfloat "f32_eq.c" "f32_eq.o"
compile_softfloat "f64_eq.c" "f64_eq.o"
compile_softfloat "f32_lt.c" "f32_lt.o"
compile_softfloat "f64_lt.c" "f64_lt.o"
compile_softfloat "f32_le.c" "f32_le.o"
compile_softfloat "f64_le.c" "f64_le.o"

# Conversions
compile_softfloat "f32_to_f64.c" "f32_to_f64.o"
compile_softfloat "f64_to_f32.c" "f64_to_f32.o"
compile_softfloat "f32_to_i32.c" "f32_to_i32.o"
compile_softfloat "f32_to_ui32.c" "f32_to_ui32.o"
compile_softfloat "f32_to_i64.c" "f32_to_i64.o"
compile_softfloat "f32_to_ui64.c" "f32_to_ui64.o"
compile_softfloat "f64_to_i32.c" "f64_to_i32.o"
compile_softfloat "f64_to_ui32.c" "f64_to_ui32.o"
compile_softfloat "f64_to_i64.c" "f64_to_i64.o"
compile_softfloat "f64_to_ui64.c" "f64_to_ui64.o"
compile_softfloat "i32_to_f32.c" "i32_to_f32.o"
compile_softfloat "ui32_to_f32.c" "ui32_to_f32.o"
compile_softfloat "i64_to_f32.c" "i64_to_f32.o"
compile_softfloat "ui64_to_f32.c" "ui64_to_f32.o"
compile_softfloat "i32_to_f64.c" "i32_to_f64.o"
compile_softfloat "ui32_to_f64.c" "ui32_to_f64.o"
compile_softfloat "i64_to_f64.c" "i64_to_f64.o"
compile_softfloat "ui64_to_f64.c" "ui64_to_f64.o"

# Internal SoftFloat helpers
compile_softfloat "s_addMagsF32.c" "s_addMagsF32.o"
compile_softfloat "s_addMagsF64.c" "s_addMagsF64.o"
compile_softfloat "s_subMagsF32.c" "s_subMagsF32.o"
compile_softfloat "s_subMagsF64.c" "s_subMagsF64.o"
compile_softfloat "s_mulAddF32.c" "s_mulAddF32.o"
compile_softfloat "s_mulAddF64.c" "s_mulAddF64.o"
compile_softfloat "s_normRoundPackToF32.c" "s_normRoundPackToF32.o"
compile_softfloat "s_normRoundPackToF64.c" "s_normRoundPackToF64.o"
compile_softfloat "s_roundPackToF32.c" "s_roundPackToF32.o"
compile_softfloat "s_roundPackToF64.c" "s_roundPackToF64.o"
compile_softfloat "s_normSubnormalF32Sig.c" "s_normSubnormalF32Sig.o"
compile_softfloat "s_normSubnormalF64Sig.c" "s_normSubnormalF64Sig.o"
compile_softfloat "s_shiftRightJam32.c" "s_shiftRightJam32.o"
compile_softfloat "s_shiftRightJam64.c" "s_shiftRightJam64.o"
compile_softfloat "s_shortShiftRightJam64.c" "s_shortShiftRightJam64.o"
compile_softfloat "s_countLeadingZeros32.c" "s_countLeadingZeros32.o"
compile_softfloat "s_countLeadingZeros64.c" "s_countLeadingZeros64.o"
compile_softfloat "s_countLeadingZeros8.c" "s_countLeadingZeros8.o"
compile_softfloat "s_approxRecip32_1.c" "s_approxRecip32_1.o"
compile_softfloat "s_approxRecipSqrt32_1.c" "s_approxRecipSqrt32_1.o"
compile_softfloat "s_approxRecip_1Ks.c" "s_approxRecip_1Ks.o"
compile_softfloat "s_approxRecipSqrt_1Ks.c" "s_approxRecipSqrt_1Ks.o"
compile_softfloat "s_roundToI32.c" "s_roundToI32.o"
compile_softfloat "s_roundToUI32.c" "s_roundToUI32.o"
compile_softfloat "s_roundToI64.c" "s_roundToI64.o"
compile_softfloat "s_roundToUI64.c" "s_roundToUI64.o"
compile_softfloat "s_addM.c" "s_addM.o"
compile_softfloat "s_subM.c" "s_subM.o"
compile_softfloat "s_mul64To128M.c" "s_mul64To128M.o"
compile_softfloat "s_shortShiftLeftM.c" "s_shortShiftLeftM.o"
compile_softfloat "s_shortShiftRightM.c" "s_shortShiftRightM.o"
compile_softfloat "s_shortShiftRightJamM.c" "s_shortShiftRightJamM.o"
compile_softfloat "s_shiftRightJamM.c" "s_shiftRightJamM.o"
compile_softfloat "s_shiftLeftM.c" "s_shiftLeftM.o"
compile_softfloat "s_negXM.c" "s_negXM.o"
compile_softfloat "s_roundMToI64.c" "s_roundMToI64.o"
compile_softfloat "s_roundMToUI64.c" "s_roundMToUI64.o"
compile_softfloat "softfloat_state.c" "softfloat_state.o"

# 8086 platform-specific files
compile_softfloat_8086 "s_propagateNaNF32UI.c" "s_propagateNaNF32UI.o"
compile_softfloat_8086 "s_propagateNaNF64UI.c" "s_propagateNaNF64UI.o"
compile_softfloat_8086 "softfloat_raiseFlags.c" "softfloat_raiseFlags.o"
compile_softfloat_8086 "s_commonNaNToF32UI.c" "s_commonNaNToF32UI.o"
compile_softfloat_8086 "s_commonNaNToF64UI.c" "s_commonNaNToF64UI.o"
compile_softfloat_8086 "s_f32UIToCommonNaN.c" "s_f32UIToCommonNaN.o"
compile_softfloat_8086 "s_f64UIToCommonNaN.c" "s_f64UIToCommonNaN.o"

echo "Creating SoftFloat library..."
# Create a library from all SoftFloat object files (excluding test.o and float.o)
riscv64-elf-ar rcs "$OUT_DIR/libsoftfloat.a" \
    "$OUT_DIR"/f32_*.o \
    "$OUT_DIR"/f64_*.o \
    "$OUT_DIR"/i32_*.o \
    "$OUT_DIR"/i64_*.o \
    "$OUT_DIR"/ui32_*.o \
    "$OUT_DIR"/ui64_*.o \
    "$OUT_DIR"/s_*.o \
    "$OUT_DIR"/softfloat_*.o

echo "Linking..."
# Link with compiler_builtins.o and -lgcc
riscv64-elf-gcc $CFLAGS -T "$SCRIPT_DIR/link.ld" \
    "$OUT_DIR/test.o" \
    "$OUT_DIR/float.o" \
    "$OUT_DIR/compiler_builtins.o" \
    "$OUT_DIR/libsoftfloat.a" \
    -lgcc \
    -o "$OUT_DIR/test.elf"

echo "Build complete: $OUT_DIR/test.elf"
echo ""
echo "Disassembly:"
riscv64-elf-objdump -S "$OUT_DIR/test.elf" > "$SCRIPT_DIR/elf.dump"
