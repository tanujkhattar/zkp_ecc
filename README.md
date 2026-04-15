# ZKP_ECC: Zero-Knowledge Proofs of Quantum Elliptic Curve Cryptography Circuits

## Overview

This repository contains Zero-Knowledge Proofs (ZKPs) generated as part of writing the paper
["Securing Elliptic Curve Cryptocurrencies against Quantum Vulnerabilities: Resource Estimates and Mitigations"](https://quantumai.google/static/site-assets/downloads/cryptocurrency-whitepaper.pdf).
The ZKPs prove that we know quantum circuits for attacking elliptic curve cryptography with 10x lower spacetime cost compared to prior art.
The ZKPs do this without revealing the contents of the circuits.
The repository also contains supporting infrastructure for generating and verifying proofs of this nature.

If you're unfamiliar with zero knowledge proofs or quantum circuits, we recommend reading [docs/getting_started.md](docs/getting_started.md).
It contains a guided walkthrough of proving and verifying the function of a simple quantum circuit (a 64 qubit adder).
It also lists crucial dependencies used by the code in this repository.

We use the [SP1 zkVM](https://github.com/succinctlabs/sp1) to generate a [Groth16](https://eprint.iacr.org/2016/260.pdf) Succinct Non-Interactive Argument of Knowledge (SNARK).
The SNARK attests to the correctness and efficiency of [elliptic curve point addition](https://en.wikipedia.org/wiki/Elliptic_curve_point_multiplication#Point_addition) circuits.
Specifically, the circuits are verified as approximately correct elliptic curve point additions using fuzz testing with cases chosen by the [Fiat-Shamir heuristic](https://en.wikipedia.org/wiki/Fiat%E2%80%93Shamir_heuristic).
(Note that Shor's algorithm uses many elliptic curve point additions, not just one.)
The circuits are specified in a custom format (see [docs/kickmix_file_format.md](docs/kickmix_file_format.md) and [docs/kickmix_instruction_set.md](docs/kickmix_instruction_set.md)), which has no support for subroutines or loops or other concepts that could make analysis non-trivial.


## Proofs

### 1. Low-Qubit Variant

The file [proofs/low_qubits/proof_9024.bin](proofs/low_qubits/proof_9024.bin) is a ZKP that we possess a kickmix circuit with the following properties:

- When run, it executes at most **2,700,000** non-Clifford gates (CCX+CCZ)
- It uses at most **1175** logical qubits
- It contains at most **17,000,000** kickmix circuit instructions
- It performs elliptic curve point addition (it passes 9024 test cases chosen randomly by the [Fiat-Shamir heuristic](https://en.wikipedia.org/wiki/Fiat%E2%80%93Shamir_heuristic)).
- It has a SHA256 hash `5373e67ca5e900819747f8c37a4a7fa9a3ea28986835436eaa9825b12a082ff2`.

The verification key for the RISC-V Elf binary of the fuzz testing program is:

> 005e287a2654d72d3c9b25ecb40772be4d3b60c2c2e009535599273746390686

The compiled RISC-V ELF binary is provided at [proofs/zkp_ecc-program](proofs/zkp_ecc-program) and the verification key is provided at [proofs/vkey.bin](proofs/vkey.bin).
The rust code used to produce the binary is in the [lib/](lib/) and [program/](program/) directories.


### 2. Low-Gate Variant

The file [proofs/low_toffoli/proof_9024.bin](proofs/low_toffoli/proof_9024.bin) is a ZKP that we possess a kickmix circuit with the following properties:

- When run, it executes at most **2,100,000** non-Clifford gates (CCX+CCZ)
- It uses at most **1425** logical qubits
- It contains at most **17,000,000** kickmix circuit instructions
- It performs elliptic curve point addition (it passes 9024 test cases chosen randomly by the [Fiat-Shamir heuristic](https://en.wikipedia.org/wiki/Fiat%E2%80%93Shamir_heuristic)).
- It has a SHA256 hash `04f17175a034cade07b0350481aab02ec4ad08254aa5d4dfd53ba217afca4f0c`.

The verification key for the RISC-V Elf binary of the fuzz testing program is:

> 005e287a2654d72d3c9b25ecb40772be4d3b60c2c2e009535599273746390686

This is the same binary as the low-qubit variant (they are simply given different inputs), and so the other details are also identical.
The compiled RISC-V ELF binary is provided at [proofs/zkp_ecc-program](proofs/zkp_ecc-program) and the verification key is provided at [proofs/vkey.bin](proofs/vkey.bin).
The rust code used to produce the binary is in the [lib/](lib/) and [program/](program/) directories.


## How We Generate the Proof

We use sp1's multi-gpu proving mode to generate proofs.
See [docs/sp1_cluster_deployment_guide.md](docs/sp1_cluster_deployment_guide.md) for more details on how to setup the sp1 cluster.
The `./run_proofs.sh` script is invoked as follows to start proof generation:

```bash
LOW_GATE_CIRCUIT_PATH=...  # you'll have to provide your own for this to work

./run_proofs.sh \
  --num-tests "9024" \
  --kmx "${LOW_GATE_CIRCUIT_PATH}" \
  --qubit-counts 1425 \
  --toffoli-counts 2100000 \
  --total-ops 17000000 \
  --proving-mode "multi-gpu"
```

and for the low-qubit variant:

```bash
LOW_QUBIT_CIRCUIT_PATH=...  # you'll have to provide your own for this to work

./run_proofs.sh \
  --num-tests "9024" \
  --kmx "${LOW_QUBIT_CIRCUIT_PATH}" \
  --qubit-counts 1175 \
  --toffoli-counts 2700000 \
  --total-ops 17000000 \
  --proving-mode "multi-gpu"
```

1. **Compilation**: The script invokes `prover/prove.rs`. Using `sp1-build`, this compiles `program/` into an ELF native to the RISC-V zkVM architecture.
2. **Private Input Injection**: The `.kmx` operations are read from disk by the host and passed as an array of private inputs into the zkVM `stdin`.
3. **Execution**: The SP1 prover natively executes the ELF, which simulates the quantum circuit. It tracks memory access, assertions, bounded limits, and computes the test evaluations.
4. **STARK Proof Generation**: The host generates a Groth16 proof and saves it to disk inside the `proofs/` directory (e.g. `proofs/low_toffoli/proof_64.bin`). The host also saves the verification key (eg: `proofs/vkey.bin`) that represents a cryptographic commitment of the exact RISC-V program that was executed in order to generate the proof.


## How to Verify a Proof

### The automatic part

After a proof is successfully created, it can be verified by a third-party observer using the standalone `verifier` binary. 

The verifier can use an explicitly provided verification key (eg: `proofs/vkey.bin`) via the `--vkey` flag, or deterministically derive the verification key from the proving ELF (eg: `proofs/zkp_ecc-program`) passed via the `--elf` flag, or the verifier can omit both flags and deterministically rebuild the ELF via Docker and derive the verification key from that.

The verifier also expects flags for the demanded resource counts. It asserts that the values committed in the proof's public outputs match these provided counts.

```bash
# Verify low toffoli proof using an explicitly exported vkey file
cargo run --release -p verifier -- \
    --proof proofs/low_toffoli/proof_9024.bin \
    --vkey proofs/vkey.bin \
    --num-tests 9024 \
    --qubit-counts 1425 \
    --toffoli-counts 2100000 \
    --total-ops 17000000
```
```bash
# Verify low qubit proof using an explicitly exported vkey file
cargo run --release -p verifier -- \
    --proof proofs/low_qubits/proof_9024.bin \
    --vkey proofs/vkey.bin \
    --num-tests 9024 \
    --qubit-counts 1175 \
    --toffoli-counts 2700000 \
    --total-ops 17000000
```

Alternatively, you can generate the verification key deterministically on-the-fly if you provide the ELF binary that was used to create the proof:

```bash
# Verify low toffoli proof by hashing the given ELF binary
cargo run --release -p verifier -- \
    --proof proofs/low_toffoli/proof_9024.bin \
    --elf proofs/zkp_ecc-program \
    --num-tests 9024 \
    --qubit-counts 1425 \
    --toffoli-counts 2100000 \
    --total-ops 17000000
```
```bash
# Verify low qubit proof by hashing the given ELF binary
cargo run --release -p verifier -- \
    --proof proofs/low_qubits/proof_9024.bin \
    --elf proofs/zkp_ecc-program \
    --num-tests 9024 \
    --qubit-counts 1175 \
    --toffoli-counts 2700000 \
    --total-ops 17000000
```

Finally, you can simply point the verifier at a proof and it will automatically construct an isolated Docker environment to deterministically rebuild the proving ELF and derive the verification key:

```bash
# Verify low toffoli proof by using Docker to rebuild the original program
cargo run --release -p verifier -- \
    --proof proofs/low_toffoli/proof_9024.bin \
    --num-tests 9024 \
    --qubit-counts 1425 \
    --toffoli-counts 2100000 \
    --total-ops 17000000
```
```bash
# Verify low qubit proof by using Docker to rebuild the original program
cargo run --release -p verifier -- \
    --proof proofs/low_qubits/proof_9024.bin \
    --num-tests 9024 \
    --qubit-counts 1175 \
    --toffoli-counts 2700000 \
    --total-ops 17000000
```

You can also run the verification script by directly providing the Groth16 proof bytes and public inputs from the paper: 

```bash
# Verify low toffoli proof with groth16 proof bytes and public inputs from the paper
cargo run --release -p verifier --  --vkey proofs/vkey.bin     --num-tests 9024     --qubit-counts 1425     --toffoli-counts 2100000     --total-ops 17000000 --groth16-proof-hex 0e78f4db0000000000000000000000000000000000000000000000000000000000000000008cd56e10c2fe24795cff1e1d1f40d3a324528d315674da45d26afb376e867000000000000000000000000000000000000000000000000000000000000000001387201c4d8f17a4582424224cf57e7df14680fc14474411475f312e65b06206112b2f47088cc51c2c924fc0008eb4ade18cb371c32211143f39b0c36b216b7b11cabe1fd8faec5702b3eabba3a306fd008cfa1c61111a47541aa233271366f51f3b47d04c9f2be8cc8427ea8052ef6ec41c24747bba26c143780d7af5873d5d20be2236503b2b7af6769f48bdd72ecf243dc650c39fe080edda195e7eaadd2b055f4262ee94c5cac9c9ad26e6072500952fbf5a48e08d07dff7790a7e7c3a0e1e7982791e47a0d8b231a9e731d890092c6b9929d987d668bd06210233d10e3411a1f4be17a13e75b0aaf29d0f5b4f05ec7cd1a8c3ef4d9d7aa16a6ad1295f3e --circuit-hash-hex 04f17175a034cade07b0350481aab02ec4ad08254aa5d4dfd53ba217afca4f0c
```

```bash
# Verify low qubit proof with groth16 proof bytes and public inputs from the paper
cargo run --release -p verifier --  --vkey proofs/vkey.bin     --num-tests 9024     --qubit-counts 1175     --toffoli-counts 2700000     --total-ops 17000000 --groth16-proof-hex 0e78f4db0000000000000000000000000000000000000000000000000000000000000000008cd56e10c2fe24795cff1e1d1f40d3a324528d315674da45d26afb376e867000000000000000000000000000000000000000000000000000000000000000000e7e4e086c9f9f4e47318d5b4925cefa0efa4853719b7c5786b5bcc4272c8c132ef1b3c2193d8ad2912a81915c8789863ba3e24bf50c88963543cba35085b1ef17eba3e7eaf1e3d628171f307bc9b2b390a297625d14df336ade99fc482a232f1b3ebf5e82075429c0d55834d1e05f555f3db6174603700e0c1275a50ee029861a098d42a49655aac19ba69cdec70d87f41d56c30711683d48cb838dbbe352cc0ffe49497d676aee03fc9e11636f456014aebb15add03831c9b624ace73dd2e9025776ccc3d1de9d8eb934f0d21eea8beb450f9544046e343d5ae83e6601763d0453613c2b1c511323c75fda5192382cbcd18902551cedd849d3125af8469fad --circuit-hash-hex 5373e67ca5e900819747f8c37a4a7fa9a3ea28986835436eaa9825b12a082ff2

```

Upon a successful invocation, the verifier prints useful information like:
1. The verification key corresponding to the ELF binary. 
2. For Groth16 or Plonk proofs, the verifier also prints bytes of the proof itself.
3. The SHA256 hash of the secret quantum circuit that was executed.
4. The demanded resource counts that the secret quantum circuit satisfies.
5. The number of test cases executed for verifying the correctness of the circuit.
6. Whether the proof is valid

### The manual part

The automatic verification checks that a given program (in this case, a quantum circuit simulator) was faithfully executed
on a set of private (known only to the prover, in this case the secret quantum circuit) and public inputs (known to both
prover and verifier, in this case the SHA256 hash of the secret quantum circuit, 9024 pseudo-random test cases and claimed resource costs).
It does not verify that the program is actually testing the correctness of quantum circuits.
It proves *that* the program output certain values, but not *why* it output those values. 
To verify the *why*, you must carefully read the source code in this repository and confirm that the program is performing
fuzz testing with inputs chosen by the [Fiat-Shamir heuristic](https://en.wikipedia.org/wiki/Fiat%E2%80%93Shamir_heuristic).
You must further verify that the implemented kickmix simulator is correct and that this is actually a valid way to certify the quantum circuits.
For example, fuzz testing can only prove *approximate* correctness and so it's crucial that Shor's algorithm tolerates approximately correct circuits.
A circuit that maps 1% of inputs to the wrong output will cause Shor's algorithm to fail around 1% of the time.
