#!/bin/bash
# Comprehensive test validation script for OpenVM RV32F Extension
# Tests all float instructions with edge cases and reports results

set -e  # Exit on error

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}OpenVM RV32F Test Validation${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Build tests
echo -e "${YELLOW}Step 1: Building tests...${NC}"
if ./build.sh > build.log 2>&1; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed - check build.log${NC}"
    exit 1
fi
echo ""

# Step 2: Run OpenVM with float tests
echo -e "${YELLOW}Step 2: Running OpenVM with float tests...${NC}"
if cargo run --release --bin cargo-openvm -- openvm run \
    --exe build/test.elf \
    --config openvm.toml \
    > test_output.log 2>&1; then
    echo -e "${GREEN}✓ OpenVM execution completed${NC}"
else
    echo -e "${RED}✗ OpenVM execution failed - check test_output.log${NC}"
    exit 1
fi
echo ""

# Step 3: Validate exit code
echo -e "${YELLOW}Step 3: Validating test results...${NC}"
if grep -q "Exit code: 0" test_output.log; then
    echo -e "${GREEN}✓ All tests passed (exit code 0)${NC}"
else
    echo -e "${RED}✗ Tests failed (non-zero exit code)${NC}"
    echo -e "${RED}Check test_output.log for details${NC}"
    exit 1
fi
echo ""

# Step 4: Count total instructions executed
echo -e "${YELLOW}Step 4: Analyzing execution metrics...${NC}"
if grep -q "Total instruction count" test_output.log; then
    INST_COUNT=$(grep "Total instruction count" test_output.log | awk '{print $NF}')
    echo -e "${BLUE}Total instructions executed: ${INST_COUNT}${NC}"
else
    echo -e "${YELLOW}Note: Could not extract instruction count from log${NC}"
fi
echo ""

# Step 5: Report which float instructions were tested
echo -e "${YELLOW}Step 5: Float instructions tested:${NC}"
echo -e "${BLUE}----------------------------------------${NC}"

# List of all RV32F instructions we're testing
declare -a float_instructions=(
    "FLW" "FSW"
    "FADD.S" "FSUB.S" "FMUL.S" "FDIV.S" "FSQRT.S"
    "FMIN.S" "FMAX.S"
    "FSGNJ.S" "FSGNJN.S" "FSGNJX.S"
    "FMADD.S" "FMSUB.S" "FNMSUB.S" "FNMADD.S"
    "FCVT.W.S" "FCVT.WU.S" "FCVT.S.W" "FCVT.S.WU"
    "FEQ.S" "FLT.S" "FLE.S"
    "FMV.X.W" "FMV.W.X"
    "FCLASS.S"
)

# Count how many unique float instructions are in the test
tested_count=0
for instr in "${float_instructions[@]}"; do
    # Extract instruction name from test.S (case insensitive search)
    if grep -qi "$instr" /home/cody/openvm/examples/floats/test.S; then
        echo -e "${GREEN}✓ ${instr}${NC}"
        ((tested_count++))
    else
        echo -e "${YELLOW}○ ${instr} (not in test.S)${NC}"
    fi
done

echo -e "${BLUE}----------------------------------------${NC}"
echo -e "${BLUE}Instructions tested: ${tested_count}/${#float_instructions[@]}${NC}"
echo ""

# Step 6: Test categories covered
echo -e "${YELLOW}Step 6: Test categories covered:${NC}"
echo -e "${BLUE}----------------------------------------${NC}"

categories_tested=0
total_categories=0

# Check for each category
if grep -q "FADD\|FSUB\|FMUL\|FDIV" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Basic arithmetic (FADD, FSUB, FMUL, FDIV)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FSQRT" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Square root (FSQRT)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FMADD\|FMSUB\|FNMSUB\|FNMADD" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Fused multiply-add (FMADD, FMSUB, FNMSUB, FNMADD)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FMIN\|FMAX" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Min/Max operations (FMIN, FMAX)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FSGNJ" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Sign injection (FSGNJ, FSGNJN, FSGNJX)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FCVT.W.S\|FCVT.WU.S\|FCVT.S.W\|FCVT.S.WU" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Float ↔ Int conversions (FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FEQ\|FLT\|FLE" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Comparisons (FEQ, FLT, FLE)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FMV.X.W\|FMV.W.X" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Register moves (FMV.X.W, FMV.W.X)${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "FCLASS" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Classification (FCLASS)${NC}"
    ((categories_tested++))
fi
((total_categories++))

# Edge case categories
if grep -q "NaN propagation\|NaN Propagation" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Edge case: NaN propagation${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "Infinity arithmetic\|Infinity Arithmetic" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Edge case: Infinity arithmetic${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "Signed zero\|Signed Zero" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Edge case: Signed zero${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "Subnormal\|subnormal" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Edge case: Subnormal numbers${NC}"
    ((categories_tested++))
fi
((total_categories++))

if grep -q "Rounding mode\|Rounding Mode" /home/cody/openvm/examples/floats/test.S; then
    echo -e "${GREEN}✓ Edge case: Rounding modes (RNE, RTZ, RDN, RUP, RMM)${NC}"
    ((categories_tested++))
fi
((total_categories++))

echo -e "${BLUE}----------------------------------------${NC}"
echo -e "${BLUE}Categories covered: ${categories_tested}/${total_categories}${NC}"
echo ""

# Step 7: Final summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ VALIDATION COMPLETE${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}All float tests passed successfully!${NC}"
echo -e "${BLUE}Instructions tested: ${tested_count}/${#float_instructions[@]}${NC}"
echo -e "${BLUE}Categories covered: ${categories_tested}/${total_categories}${NC}"
echo ""
echo -e "${YELLOW}Detailed logs:${NC}"
echo -e "  - Build log: build.log"
echo -e "  - Execution log: test_output.log"
echo ""

exit 0
