use clap::Parser;
use sp1_sdk::{Elf, HashableKey, Prover, ProverClient, ProvingKey, SP1ProofWithPublicValues, SP1PublicValues};
use sp1_build::{build_program_with_args, BuildArgs};

use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    proof: Option<String>,

    #[arg(long)]
    groth16_proof_hex: Option<String>,

    #[arg(long)]
    circuit_hash_hex: Option<String>,

    #[arg(long)]
    vkey: Option<String>,

    #[arg(long)]
    elf: Option<String>,

    #[arg(long)]
    qubit_counts: u64,

    #[arg(long)]
    toffoli_counts: u64,

    #[arg(long)]
    total_ops: u64,

    #[arg(long)]
    num_tests: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = ProverClient::from_env().await;

    // Read or generate the verification key.
    let vk = if let Some(vkey_path) = args.vkey {
        let vk_bytes = std::fs::read(&vkey_path).expect("failed to read vkey file");
        bincode::deserialize(&vk_bytes).expect("failed to deserialize vkey")
    } else if let Some(elf_path) = args.elf {
        let bytes = std::fs::read(&elf_path).expect("failed to read elf file");
        let elf_data = Elf::Dynamic(Arc::from(bytes.into_boxed_slice()));
        let pk = client.setup(elf_data).await.expect("failed to setup");
        pk.verifying_key().clone()
    } else {
        println!("Neither --vkey nor --elf provided. Building program using Docker...");
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
        let program_dir = workspace_root.join("program");

        let build_args = BuildArgs {
            docker: true,
            ..Default::default()
        };
        build_program_with_args(program_dir.to_str().unwrap(), build_args);
        
        let elf_path = workspace_root.join("target/elf-compilation/docker/riscv64im-succinct-zkvm-elf/release/zkp_ecc-program");
        let bytes = std::fs::read(&elf_path).unwrap_or_else(|e| {
            panic!("failed to read generated elf file from {}: {}", elf_path.display(), e)
        });
        let elf_data = Elf::Dynamic(Arc::from(bytes.into_boxed_slice()));
        let pk = client.setup(elf_data).await.expect("failed to setup");
        pk.verifying_key().clone()
    };

    println!("Verifying Key (Hex): {}", vk.bytes32());

    if let Some(proof_path) = args.proof {
        let mut proof = SP1ProofWithPublicValues::load(&proof_path).expect("failed to load proof");
        if matches!(&proof.proof, sp1_sdk::SP1Proof::Plonk(_) | sp1_sdk::SP1Proof::Groth16(_)) {
            println!("Proof (Hex): {}", hex::encode(proof.bytes()));
        }

        client.verify(&proof, &vk, None).expect("failed to verify proof");

        println!("✅ Proof verified successfully using ProverClient.");

        // Read and print public values in human-readable format
        let output_hash = proof.public_values.read::<[u8; 32]>();
        println!("Circuit hash commitment: {}", hex::encode(output_hash));

        let num_tests = proof.public_values.read::<u64>();
        println!("Demanded Number of tests: {}", num_tests);

        let demanded_qubit_count = proof.public_values.read::<u64>();
        println!("Demanded Qubit count: {}", demanded_qubit_count);

        let demanded_average_non_clifford_count = proof.public_values.read::<u64>();
        println!("Demanded Average non-Clifford count: {}", demanded_average_non_clifford_count);

        let demanded_total_ops = proof.public_values.read::<u64>();
        println!("Demanded Total ops: {}", demanded_total_ops);

        // Assert that proof values satisfy the passed demands.
        assert!(num_tests == args.num_tests, "Failed to verify: num_tests not satisfied by proof");
        assert!(demanded_qubit_count == args.qubit_counts, "Failed to verify: qubit_counts not satisfied by proof");
        assert!(demanded_average_non_clifford_count == args.toffoli_counts, "Failed to verify: toffoli_counts not satisfied by proof");
        assert!(demanded_total_ops == args.total_ops, "Failed to verify: total_ops not satisfied by proof");

        println!("✅ Proof passed demand checks.");
    } else if let Some(groth16_proof_hex) = args.groth16_proof_hex {
        let circuit_hash_hex = args.circuit_hash_hex.expect("circuit_hash_hex is required for groth16_proof_hex mode");

        let mut public_values = SP1PublicValues::new();
        let circuit_hash_bytes: [u8; 32] = hex::decode(&circuit_hash_hex)
            .expect("Invalid circuit hash hex")
            .try_into()
            .expect("Invalid circuit hash length");
        public_values.write(&circuit_hash_bytes);
        public_values.write(&args.num_tests);
        public_values.write(&args.qubit_counts);
        public_values.write(&args.toffoli_counts);
        public_values.write(&args.total_ops);
        let sp1_public_inputs_bytes = public_values.to_vec();

        let proof_bytes = hex::decode(&groth16_proof_hex).expect("Invalid proof hex");

        println!("Verifying proof using sp1_verifier::Groth16Verifier...");

        sp1_verifier::Groth16Verifier::verify(&proof_bytes, &sp1_public_inputs_bytes, &vk.bytes32(), &sp1_verifier::GROTH16_VK_BYTES)
            .expect("failed to verify proof with Groth16Verifier");

        println!("✅ Proof verified successfully using Groth16Verifier.");
    } else {
        panic!("Error: You must provide either --proof or --groth16-proof-hex");
    }
}