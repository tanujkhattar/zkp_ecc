use zkp_ecc_lib::circuit::{QubitId, BitId};
use zkp_ecc_lib::Simulator;
use zkp_ecc_lib::Circuit;
use sha3::{Shake256, digest::ExtendableOutput};

#[test]
fn test_conditional_reset() {
    let circuit = Circuit::from_text("
        BIT_STORE0 b0
        X q0
        R q0 if b0
        X q0
    ");
    let hasher = Shake256::default();
    let mut xof = hasher.finalize_xof();
    let mut sim = Simulator::new(
        circuit.num_qubits as usize,
        circuit.num_bits as usize,
        &mut xof,
    );
    sim.clear_for_shot();
    sim.apply_iter(circuit.operations.iter());
    assert_eq!(sim.qubit(QubitId(0)), 0);
    assert_eq!(sim.bit(BitId(0)), 0);
}

#[test]
fn test_push_conditional_reset() {
    let circuit = Circuit::from_text("
        BIT_STORE0 b0
        X q0
        PUSH_CONDITION if b0
        R q0
        POP_CONDITION
        X q0
    ");
    let hasher = Shake256::default();
    let mut xof = hasher.finalize_xof();
    let mut sim = Simulator::new(
        circuit.num_qubits as usize,
        circuit.num_bits as usize,
        &mut xof,
    );
    sim.clear_for_shot();
    sim.apply_iter(circuit.operations.iter());
    assert_eq!(sim.qubit(QubitId(0)), 0);
    assert_eq!(sim.bit(BitId(0)), 0);
}

#[test]
fn test_conditional_hmr() {
    let circuit = Circuit::from_text("
        BIT_STORE0 b0
        X q0
        HMR q0 b1 if b0
        X q0
    ");
    let hasher = Shake256::default();
    let mut xof = hasher.finalize_xof();
    let mut sim = Simulator::new(
        circuit.num_qubits as usize,
        circuit.num_bits as usize,
        &mut xof,
    );
    sim.clear_for_shot();
    sim.apply_iter(circuit.operations.iter());
    assert_eq!(sim.qubit(QubitId(0)), 0);
    assert_eq!(sim.bit(BitId(0)), 0);
    assert_eq!(sim.bit(BitId(1)), 0);
}

#[test]
fn test_conditional_hmr_bit1() {
    let circuit = Circuit::from_text("
        BIT_STORE0 b0
        BIT_STORE1 b1
        X q0
        HMR q0 b1 if b0
        X q0
    ");
    let hasher = Shake256::default();
    let mut xof = hasher.finalize_xof();
    let mut sim = Simulator::new(
        circuit.num_qubits as usize,
        circuit.num_bits as usize,
        &mut xof,
    );
    sim.clear_for_shot();
    sim.apply_iter(circuit.operations.iter());
    assert_eq!(sim.qubit(QubitId(0)), 0);
    assert_eq!(sim.bit(BitId(0)), 0);
    assert_eq!(sim.bit(BitId(1)), !0);
}

#[test]
fn test_push_conditional_hmr() {
    let circuit = Circuit::from_text("
        BIT_STORE0 b0
        X q0
        PUSH_CONDITION if b0
        HMR q0 b1
        POP_CONDITION
        X q0
    ");
    let hasher = Shake256::default();
    let mut xof = hasher.finalize_xof();
    let mut sim = Simulator::new(
        circuit.num_qubits as usize,
        circuit.num_bits as usize,
        &mut xof,
    );
    sim.clear_for_shot();
    sim.apply_iter(circuit.operations.iter());
    assert_eq!(sim.qubit(QubitId(0)), 0);
    assert_eq!(sim.bit(BitId(0)), 0);
    assert_eq!(sim.bit(BitId(1)), 0);
}
