#[cfg(test)]
mod tests {
    use eyre::Result;
    use openvm_rv32f_transpiler::Rv32FArchATranspilerExtension;
    use openvm_rv32im_transpiler::{Rv32ITranspilerExtension, Rv32MTranspilerExtension};
    use openvm_stark_sdk::p3_baby_bear::BabyBear;
    use openvm_transpiler::{transpiler::Transpiler, TranspilerExtension};

    type F = BabyBear;

    // Test that the transpiler can be instantiated with F extension
    #[test]
    fn test_transpiler_with_f_extension() -> Result<()> {
        let _transpiler = Transpiler::<F>::default()
            .with_extension(Rv32ITranspilerExtension)
            .with_extension(Rv32MTranspilerExtension)
            .with_extension(Rv32FArchATranspilerExtension);

        // If we get here, the transpiler was created successfully
        Ok(())
    }

    // Test transpiling a manually constructed F instruction stream
    #[test]
    fn test_transpile_fadd_instruction() -> Result<()> {
        let ext = Rv32FArchATranspilerExtension;

        // FADD.S f3, f1, f2 (manually encoded)
        // opcode=0x53, rd=3, funct3=0, rs1=1, rs2=2, funct7=0x00
        let fadd_inst = 0x002081D3u32;

        let result: Option<openvm_transpiler::TranspilerOutput<F>> =
            ext.process_custom(&[fadd_inst]);
        assert!(result.is_some(), "FADD.S should be recognized");

        let output = result.unwrap();
        assert_eq!(output.used_u32s, 1, "Should consume 1 u32");
        assert_eq!(output.instructions.len(), 10, "Should emit 10 instructions");

        Ok(())
    }

    // Test that non-F instructions are not processed
    #[test]
    fn test_non_f_instruction_ignored() -> Result<()> {
        let ext = Rv32FArchATranspilerExtension;

        // ADD x3, x1, x2 (RV32I instruction, not F extension)
        // opcode=0x33, not 0x53
        let add_inst = 0x002081B3u32;

        let result: Option<openvm_transpiler::TranspilerOutput<F>> =
            ext.process_custom(&[add_inst]);
        assert!(result.is_none(), "Non-F instruction should not be processed");

        Ok(())
    }

    // TODO: Full integration tests require:
    // 1. openvm-rv32f-circuit (for VM execution)
    // 2. openvm-rv32f-guest (for compiling guest programs)
    // 3. Runtime library compiled for RISC-V target
    // 4. Test guest programs in programs/examples/
    //
    // Once those are available, add tests like:
    // - test_float_add: Compile program that does 1.0 + 2.0, execute, verify result
    // - test_float_mul: Test multiplication
    // - test_float_cmp: Test comparisons (FEQ, FLT, FLE)
    // - test_float_cvt: Test conversions between float and int
}
