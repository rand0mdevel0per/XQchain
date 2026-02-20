# XCQA Chain

A quantum-resistant blockchain with integrated compute and storage infrastructure.

## Features

- **Quantum-Resistant Cryptography**
  - XCQA signatures (7+ layer encryption)
  - ML-DSA-65 (FIPS 204) post-quantum signatures
  - Lattice-based commitments (Module-SIS)
  - Lattice range proofs for confidential transactions

- **Proof-of-Work Mining**
  - CPU and GPU solver support
  - Dynamic difficulty adjustment
  - XCQA-based PoW algorithm

- **Privacy-Preserving Transactions**
  - Confidential amounts with range proofs
  - Lattice-based homomorphic commitments

## Architecture

```
xcqa-chain/
├── crates/
│   ├── xcqa-crypto/      # Core cryptographic primitives
│   ├── xcqa-chain-core/  # Blockchain data structures
│   ├── xcqa-chain-pow/   # Proof-of-Work solvers
│   ├── xcqa-chain-net/   # P2P networking
│   └── xcqa-chain-node/  # Full node implementation
```

## Building

```bash
cargo build --release
```

## Running a Node

```bash
# Run node without mining
cargo run --release -p xcqa-chain-node

# Run node with CPU mining
cargo run --release -p xcqa-chain-node -- --mine

# Run node with GPU mining
cargo run --release -p xcqa-chain-node -- --mine --gpu --gpu-vram-mb 2048

# Custom port and layers
cargo run --release -p xcqa-chain-node -- --port 8333 --layers 8 --mine
```

## License

MIT
