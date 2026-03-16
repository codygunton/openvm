/// Generates OpenVM proofs and writes them to disk for size measurement.
///
/// Usage:
///   cargo run -p openvm-benchmarks-prove --release --bin proof_size -- <PROGRAM> [OPTIONS]
///   cargo run -p openvm-benchmarks-prove --release --features cuda --bin proof_size -- <PROGRAM> [OPTIONS]
///
/// Programs:
///   fibonacci       Fibonacci (n=100k), uses fibonacci/openvm.toml config
///   revm-transfer   100 EVM value transfers, uses standard SDK config (secp256k1/ECC)
use std::{fs, path::PathBuf, time::Instant};

use clap::{Parser, ValueEnum};
use eyre::Result;
use openvm_benchmarks_prove::util::BenchmarkCli;
use openvm_benchmarks_utils::{get_programs_dir, read_elf_file};
use openvm_sdk::{codec::Encode, config::SdkVmConfig, Sdk, StdIn};

#[derive(Clone, ValueEnum, Debug)]
enum Program {
    Fibonacci,
    RevmTransfer,
}

#[derive(Parser, Debug)]
#[command(about = "Generate OpenVM proofs and write to disk")]
struct Args {
    /// Which guest program to prove
    program: Program,

    #[command(flatten)]
    bench: BenchmarkCli,

    /// Rebuild guest ELF from source instead of using prebuilt
    #[arg(long)]
    rebuild: bool,

    /// Also generate the aggregated STARK proof (much slower)
    #[arg(long)]
    aggregate: bool,

    /// Directory to write proof files
    #[arg(short, long, default_value = "proof-output")]
    output_dir: PathBuf,

    /// Fibonacci iteration count (only for fibonacci program)
    #[arg(long, default_value_t = 100_000)]
    fib_n: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output_dir = &args.output_dir;
    fs::create_dir_all(output_dir)?;

    let (program_name, guest_name, config, stdin) = match args.program {
        Program::Fibonacci => {
            let config =
                SdkVmConfig::from_toml(include_str!("../../../guest/fibonacci/openvm.toml"))?
                    .app_vm_config;
            let mut stdin = StdIn::default();
            stdin.write(&args.fib_n);
            (
                format!("fibonacci (n = {})", args.fib_n),
                "fibonacci",
                config,
                stdin,
            )
        }
        Program::RevmTransfer => {
            // Standard SDK config includes secp256k1 moduli/ECC needed by revm
            let config = SdkVmConfig::from_toml(include_str!(
                "../../../../crates/sdk/src/config/openvm_standard.toml"
            ))?
            .app_vm_config;
            (
                "revm_transfer (100 EVM value transfers)".to_string(),
                "revm_transfer",
                config,
                StdIn::default(),
            )
        }
    };

    let elf = if args.rebuild {
        eprintln!("Rebuilding guest ELF from source...");
        args.bench.build_bench_program(guest_name, &config, None)?
    } else {
        let elf_dir = get_programs_dir().join(guest_name).join("elf");
        let elf_path = fs::read_dir(&elf_dir)?
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().is_some_and(|ext| ext == "elf"))
            .map(|e| e.path())
            .ok_or_else(|| eyre::eyre!("No prebuilt ELF in {}, use --rebuild", elf_dir.display()))?;
        eprintln!("Using prebuilt ELF: {}", elf_path.display());
        read_elf_file(&elf_path)?
    };
    let app_config = args.bench.app_config(config);
    let sdk = Sdk::new(app_config)?;

    eprintln!("Proving: {program_name}");
    let t0 = Instant::now();
    let mut app_prover = sdk.app_prover(elf.clone())?.with_program_name(guest_name);
    let app_proof = app_prover.prove(stdin)?;
    let app_prove_time = t0.elapsed();
    eprintln!("Proved in {app_prove_time:.1?} ({} segments)", app_proof.per_segment.len());

    // Write full app proof (ContinuationVmProof = all segments + public values)
    fs::write(output_dir.join("app_proof.bin"), app_proof.encode_to_vec()?)?;

    // Aggregated STARK proof
    if args.aggregate {
        eprintln!("Generating aggregated STARK proof...");
        let t1 = Instant::now();
        let mut prover = sdk.prover(elf)?.with_program_name(guest_name);
        let stark_proof = prover.prove(StdIn::default())?;
        eprintln!("Aggregated in {:.1?}", t1.elapsed());
        fs::write(
            output_dir.join("stark_proof.bin"),
            stark_proof.encode_to_vec()?,
        )?;
    }

    Ok(())
}
