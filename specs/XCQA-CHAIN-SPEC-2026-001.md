# XCQA-CHAIN TECHNICAL SPECIFICATION

**Document Number:** XCQA-CHAIN-SPEC-2026-001  
**Version:** 0.2.0-draft  
**Status:** Draft  
**Language:** English  
**Implementation Target:** Rust (stable, edition 2021)  
**Date:** 2026-02-20  

---

## Table of Contents

1. Scope
2. Normative References
3. Terms, Definitions, and Abbreviations
4. System Architecture Overview
5. Cryptographic Primitives
6. Identity and Key Management
7. Consensus Layer (PoW)
8. Block and Transaction Structure
9. Account Model and State
10. Smart Contract Layer (EVM)
11. Compute Layer (Firecracker/QEMU)
12. Storage Layer (DHT/RAID6)
13. Privacy Transfer Protocol
14. Network Protocol
15. Fee and Incentive Model
16. Security Considerations
17. Rust Implementation Guidelines
18. Chain Compression and Historical Archival Protocol

---

## 1. Scope

This specification defines the XCQA-Chain distributed ledger system, a quantum-resistant, privacy-capable blockchain with integrated compute and storage infrastructure. The system is designed for implementation in Rust and SHALL support Linux and Windows host environments.

The specification covers:

- Cryptographic primitives and their usage constraints
- Consensus mechanism based on XCQA dictionary collision proof-of-work
- Account model with dual-key identity (ML-DSA-65 + XCQA)
- EVM-compatible deterministic smart contracts
- Firecracker (Linux) / QEMU (Windows) microVM compute layer
- RAID6+DHT distributed storage layer
- Zero-knowledge privacy transfer protocol based on XCQA signatures

Systems implementing this specification MUST comply with all normative requirements (indicated by SHALL, MUST, MUST NOT). Recommended practices are indicated by SHOULD. Optional features are indicated by MAY.

---

## 2. Normative References

- **XCQA**: XC Quick Algo cryptosystem, version 0.1.0 (see SECURITY_ANALYSIS.md)
- **ML-DSA-65**: FIPS 204, Module-Lattice-Based Digital Signature Standard, Security Category 3
- **BLAKE3**: BLAKE3 cryptographic hash function (128-byte output mode for BLAKE3-512)
- **HKDF**: RFC 5869, HMAC-based Key Derivation Function
- **Pedersen Commitment**: Standard EC Pedersen commitment over Ristretto255
- **Bulletproofs**: Efficient range proofs over Pedersen commitments
- **Fiat-Shamir**: Transformation of interactive proofs to non-interactive (CRYPTO 1986)
- **Merkle Tree**: Binary Merkle tree with BLAKE3 node hashing
- **UUIDv7**: RFC 9562, Universally Unique Identifier Version 7
- **EVM**: Ethereum Virtual Machine Yellow Paper specification
- **RAID6**: Reed-Solomon erasure coding, 2 parity shards
- **DHT**: Kademlia distributed hash table (XOR metric)
- **Firecracker**: AWS Firecracker VMM v1.x (Linux only)
- **QEMU**: QEMU microvm machine type (Windows fallback)
- **Zstd**: RFC 8478, Zstandard compression

---

## 3. Terms, Definitions, and Abbreviations

### 3.1 Terms and Definitions

**Account**: A network participant identified by a UUIDv7 address, holding two key pairs (ML-DSA-65, XCQA) and a balance expressed as a Pedersen Commitment.

**Block**: An ordered collection of transactions, new-user registrations, and consensus metadata, cryptographically linked to the preceding block.

**Commitment**: A Pedersen commitment `C = v·G + r·H` over the Ristretto255 group, hiding value `v` with blinding factor `r`.

**Firecracker VM**: A lightweight virtual machine instance executed on Linux hosts via the Firecracker VMM.

**Inspector**: An independently assigned observer node that monitors resource consumption of a compute task via the microVM's built-in monitor interface.

**Microservice**: A containerized application packaged as a rootfs image with a manifest, executed inside a MicroVM.

**Nullifier**: A value `N = H(sk_xcqa || commitment)` used to prevent double-spending in the privacy pool without revealing identity.

**PoW Puzzle**: The XCQA private-key collision problem for a given epoch public key dictionary, combined with a hash difficulty target on the block nonce.

**Privacy Pool**: An on-chain structure holding encrypted fund commitments that can be claimed via zero-knowledge proofs without linking sender to receiver.

**QEMU MicroVM**: A lightweight QEMU instance using the `microvm` machine type, used as a Firecracker substitute on Windows hosts.

**UUIDv7**: A time-ordered UUID with 48-bit timestamp prefix, used as wallet address.

**vNode**: A virtual node in the consistent hash ring used for compute and storage assignment.

**ZK Proof**: A non-interactive zero-knowledge proof produced via Fiat-Shamir transformation over XCQA and Pedersen constraints.

### 3.2 Abbreviations

| Abbreviation | Expansion |
|---|---|
| DHT | Distributed Hash Table |
| EVM | Ethereum Virtual Machine |
| HKDF | HMAC-based Key Derivation Function |
| ML-DSA | Module-Lattice Digital Signature Algorithm |
| PoW | Proof of Work |
| RAID6 | Redundant Array of Independent Disks, Level 6 |
| TXID | Transaction Identifier |
| VMM | Virtual Machine Monitor |
| ZK | Zero-Knowledge |

---

## 4. System Architecture Overview

### 4.1 Layered Architecture

XCQA-Chain is organized into four cooperating layers:

```
┌──────────────────────────────────────────────────────────┐
│  Layer 0: Consensus & Identity                           │
│  XCQA PoW · ML-DSA-65 Identity · UUIDv7 Addressing      │
├──────────────────────────────────────────────────────────┤
│  Layer 1: EVM Chain (Deterministic)                      │
│  Transfers · Smart Contracts · Privacy Pool · Fees       │
├──────────────────────────────────────────────────────────┤
│  Layer 2: Compute (Non-Deterministic)                    │
│  Firecracker (Linux) / QEMU MicroVM (Windows)            │
│  Consistent Hash Routing · Inspector · Keyring           │
├──────────────────────────────────────────────────────────┤
│  Layer 3: Storage (Distributed)                          │
│  RAID6 · DHT (Kademlia) · Keyring Encryption             │
└──────────────────────────────────────────────────────────┘
```

### 4.2 Cross-Layer Interactions

- Layer 0 provides finality guarantees to Layer 1.
- Layer 1 controls fee escrow, task dispatch, and result settlement for Layers 2 and 3.
- Layer 2 reads and writes state to Layer 3 via the keyring-encrypted DHT.
- Layer 2 reports resource usage to Layer 1 via the Inspector mechanism.

### 4.3 Host Platform Variants

| Feature | Linux Host | Windows Host |
|---|---|---|
| MicroVM Backend | Firecracker v1.x | QEMU microvm |
| KVM Acceleration | Required | Optional (WHPX/HAXM) |
| Network Backend | virtio-net TAP | QEMU user-mode net |
| Block Backend | virtio-blk | QEMU virtio-blk |

All Rust crates MUST compile on both platforms. Platform-specific VMM code SHALL be gated behind `#[cfg(target_os)]` or feature flags.

---

## 5. Cryptographic Primitives

### 5.1 Hash Functions

#### 5.1.1 BLAKE3-512

Used for: block header hashing, Merkle tree nodes, PoW seed derivation.

```
BLAKE3-512(data) → [u8; 64]
```

Output length SHALL be 64 bytes (512 bits) via BLAKE3 extended output (XOF mode).

Rust crate: `blake3` with `no_std` compatible configuration.

#### 5.1.2 SHA-512

Used for: TXID computation, registration commitment, all security-critical 32-byte digests.

```
SHA512(data) → [u8; 64]
```

SHA-256 SHALL NOT be used for any security-critical purpose. Against a quantum adversary with Grover's algorithm, SHA-256 provides only 64-bit effective security; SHA-512 provides 256-bit effective security.

Rust crate: `sha2`.

#### 5.1.3 BLAKE3 as Fiat-Shamir Oracle

Challenge derivation in ZK proofs SHALL use BLAKE3 in 512-bit output mode:

```
challenge = BLAKE3-512(commitment_bytes || public_inputs || block_hash)
```

BLAKE3-512 provides 256-bit post-quantum security (Grover halves to 256 bits effective).

### 5.2 Key Derivation

#### 5.2.1 HKDF-SHA256

Used for: PoW epoch key expansion.

```
HKDF-Extract(salt, ikm) → prk
HKDF-Expand(prk, info, length) → okm
```

Parameters for PoW seed expansion:

```
salt = BLAKE3-512(prev_block_header)
ikm  = block_height_le64 || difficulty_le32
info = b"XCQA-CHAIN-POW-EPOCH-V1"
length = required_dict_bytes
```

### 5.3 XCQA Cryptosystem

#### 5.3.1 Overview

XCQA is used in two distinct roles:

1. **PoW Puzzle**: Epoch public key dictionaries are generated from HKDF output; miners search for the corresponding private key parameters.
2. **Privacy ZK Witness**: Per-account XCQA key pairs serve as ZK witnesses in privacy transfer proofs, reducing circuit complexity vs. ML-DSA-65.

#### 5.3.2 Dictionary Layer Configuration

For PoW (security-critical), the minimum layer count SHALL be 8. The recommended default is 8 layers.

```
Layer 0: 8  bits input → 12 bits output  (256 entries)
Layer 1: 6  bits input →  9 bits output  ( 64 entries)
Layer 2: 4  bits input →  6 bits output  ( 16 entries)
Layer 3: 2  bits input →  4 bits output  (  4 entries)
... layers 4-7: repeat pattern with distinct random permutations
```

For privacy XCQA key pairs (per-account), layer count SHALL be at least 7.

#### 5.3.3 PoW Epoch Key Generation

```
epoch_bytes = HKDF(
    salt = BLAKE3-512(prev_header),
    info = b"XCQA-CHAIN-POW-EPOCH-V1",
    length = total_dict_value_bytes
)

For each layer L, entry E:
    value_bits = epoch_bytes[offset..offset+value_width]
    PK_dict[L][sequential_key_E] = value_bits
    offset += value_width
```

The resulting public key dictionary is determined entirely by the previous block and is globally verifiable. The private key (transformation parameters `θ = (shift, rotate, xor_mask, ...)`) MUST be found by exhaustive search.

#### 5.3.4 Hardness Guarantee

At layer count n > 7.42 (practically: n ≥ 8):

- Search space growth rate >> solution set growth rate
- Solutions are uniformly distributed in parameter space (no structural bias)
- Inter-layer bit overlap prevents MITM decomposition
- Sequential data dependencies prevent Grover oracle construction
- Random memory access pattern resists ASIC optimization

#### 5.3.5 Privacy XCQA Key Generation

Per-account XCQA key pairs are generated independently of PoW:

```rust
fn xcqa_keygen(rng: &mut impl CryptoRng) -> (XcqaPublicKey, XcqaPrivateKey) {
    let theta = TransformParams::random(rng);  // (shift, rotate, xor_mask, ...)
    let base_dict = generate_base_dict(rng);
    let pk = apply_permutation(base_dict, theta);
    let sk = XcqaPrivateKey { theta, base_dict };
    (pk, sk)
}
```

#### 5.3.6 XCQA ZK Signature (Commitment-Challenge-Response)

```
Sign(message, sk, pk):
    nonce    = random_bytes(aligned_to_encoding_cycles)
    commitment = XCQA_Encrypt(nonce, pk)
    challenge  = BLAKE3(commitment || message || block_hash)
    response   = XCQA_Decrypt(commitment, sk) XOR challenge
    return Signature { commitment, response }

Verify(message, sig, pk):
    challenge  = BLAKE3(sig.commitment || message || block_hash)
    recovered  = sig.response XOR challenge
    return XCQA_Encrypt(recovered, pk) == sig.commitment
```

The ZK property holds because: verification requires only `pk` and `commitment`; `sk` (the transform parameters `θ`) is never exposed; and `challenge` is derived non-interactively via Fiat-Shamir.

### 5.4 ML-DSA-65

Used for: all on-chain identity binding, transaction authorization, authorized debit pre-signatures.

- Security category: NIST Level 3
- Public key size: 1952 bytes
- Private key size: 4000 bytes
- Signature size: 3309 bytes

Rust crate: `fips204` or `ml-dsa` (FIPS 204 compliant).

All ML-DSA-65 operations SHALL use the deterministic signing variant.

### 5.5 Lattice-Based Commitments (Module-SIS)

Elliptic-curve Pedersen commitments are vulnerable to Shor's algorithm. XCQA-Chain uses lattice-based commitments with hardness reduction to Module-SIS, sharing parameters with ML-DSA-65.

#### 5.5.1 Parameters

Reuse ML-DSA-65 lattice parameters to avoid new security assumptions:

```
Ring:    R_q = Z_q[X] / (X^256 + 1)
Modulus: q = 8380417  (23-bit prime)
Rank:    k = 4, l = 4  (as in ML-DSA-65 level 3)
```

#### 5.5.2 Public Commitment Matrix

The commitment matrix `A ∈ R_q^{k×l}` is generated once at genesis via verifiable random generation (no trusted setup):

```
A = expand_matrix(BLAKE3-512(b"XCQA-CHAIN-COMMIT-MATRIX-V1" || genesis_hash))
```

Any party can reproduce `A` from the genesis block hash. No trusted party is required.

#### 5.5.3 Commitment Scheme

```
Gadget vector: G ∈ R_q^k  (standard binary gadget decomposition)

Commit(v: u64, r: R_q^l) → R_q^k:
    C = A · r + v · G   (mod q)

where:
    v is the committed value (encoded as constant polynomial v·1 ∈ R_q)
    r is the randomness vector (uniformly random small coefficients)
```

#### 5.5.4 Homomorphic Properties

```
Commit(v1, r1) + Commit(v2, r2) = Commit(v1+v2, r1+r2)   ✓
Commit(v1, r1) - Commit(v2, r2) = Commit(v1-v2, r1-r2)   ✓
```

These properties are used identically to Pedersen commitments in all balance arithmetic. The circuit constraint structure (§13.4.2) is unchanged; only the underlying group operations differ.

#### 5.5.5 Binding and Hiding

- **Computationally binding**: Finding two openings for the same commitment reduces to Module-SIS (same hardness as ML-DSA-65 forgery).
- **Statistically hiding**: `r` is sampled from a distribution with sufficient entropy; commitment reveals no information about `v` unconditionally.

#### 5.5.6 Serialization

A commitment `C ∈ R_q^k` is serialized as `k × 256 × ceil(log2(q)/8)` bytes = `k × 256 × 3` bytes = **3072 bytes** for k=4.

Rust implementation: within `xcqa-crypto` crate, module `lattice_commit`. No external crate available; implement using `ml-dsa` lattice arithmetic primitives.

### 5.6 Lattice Range Proofs (BDLOP + Compressed Σ-Protocol)

Bulletproofs depend on elliptic curve discrete log hardness and are not quantum-resistant. XCQA-Chain uses lattice-based range proofs.

#### 5.6.1 Scheme Overview

To prove `v ∈ [0, 2^64)` given commitment `C = Commit(v, r)`:

```
1. Bit decomposition:
     v = Σ_{i=0}^{63} b_i · 2^i
     where b_i ∈ {0, 1}

2. Commit to each bit:
     C_i = Commit(b_i, r_i)   for i = 0..63

3. For each C_i, prove b_i ∈ {0,1} using a Σ-protocol:
     b_i(b_i - 1) = 0   ⟺   b_i is a bit
     This translates to a lattice AND-relation proof.

4. Prove linear combination:
     Σ b_i · 2^i = v
     ⟺  Σ C_i · 2^i = C   (by commitment homomorphism)
     This is a deterministic check, no additional ZK needed.

5. Fiat-Shamir transform all Σ-protocols:
     challenge = BLAKE3-512(all_commitments || public_inputs || block_hash)
```

#### 5.6.2 Proof Size

```
Per-bit proof (Σ-protocol):  ~2 × commitment_size = 2 × 3072 = 6144 bytes
Bit commitments:             64 × 3072             = 196608 bytes
Total per range proof:       ~200 KB
```

This is larger than Bulletproofs (~672 bytes) but provides full quantum resistance. For bandwidth optimization, range proofs SHOULD be compressed with zstd before transmission (typical compression ratio 3–5×, yielding ~50 KB effective size).

#### 5.6.3 Aggregated Range Proofs

When multiple range proofs are required in a single transaction (e.g., `amount`, `fee`, `remainder`), they SHOULD be aggregated:

```
Aggregate proof size ≈ single_proof_size × log2(n_proofs) / n_proofs
```

Aggregation is performed by batching all Σ-protocol commitments before the single Fiat-Shamir challenge.

#### 5.6.4 AmountLeqProof

To prove `actual ≤ max` (used in authorized debits):

```
diff = max - actual   (both committed)
Prove: diff ∈ [0, 2^64)   via standard range proof above
Verify: Commit(actual, r_a) + Commit(diff, r_d) == Commit(max, r_m)
```

Rust implementation: within `xcqa-crypto` crate, module `lattice_range`. Implement using `ml-dsa` ring arithmetic.

---

## 6. Identity and Key Management

### 6.1 Account Structure

Each account consists of:

```rust
pub struct Account {
    pub uuid:          Uuid,
    pub pk_mldsa:      MlDsa65PublicKey,   // 1952 bytes, on-chain
    pub pk_xcqa:       XcqaPublicKey,      // variable, on-chain
    pub balance_comm:  LatticeCommitment,  // Module-SIS commitment to balance
    pub balance_rand:  LatticeRandVec,     // randomness vector r, kept by owner
    pub nonce:         u64,
    pub reputation:    i64,
    pub registered_at: BlockHeight,
}
```

Balance blinding factors MUST be stored locally by the account owner and MUST NOT be transmitted on-chain.

### 6.2 UUID Generation

UUIDs SHALL be self-generated by the registering node using UUIDv7:

```
uuid = UUIDv7(timestamp_ms, random_a, random_b)
```

UUID uniqueness is enforced on-chain: if two registrations submit the same UUID, the first to be included in a finalized block takes precedence. The second SHALL be rejected.

### 6.3 Registration Transaction

#### 6.3.1 TXID Computation

```
reg_payload = uuid_bytes || pk_mldsa_bytes || pk_xcqa_bytes || timestamp_le64
TXID = SHA512(reg_payload || latest_finalized_block_hash)
```

Including `latest_finalized_block_hash` bounds the registration to a time window, preventing pre-computation attacks on the registration PoW.

The registration PoW target MUST use a block hash no older than `REGISTRATION_EXPIRY_BLOCKS = 10` blocks.

#### 6.3.2 Registration PoW

The registrant SHALL solve a reduced-difficulty XCQA PoW over the TXID:

```
reg_epoch_pk = HKDF(
    salt = TXID,
    info = b"XCQA-CHAIN-REG-V1",
    length = dict_bytes_for_layer_count(REG_POW_LAYERS)
)

Find SK_reg such that:
    XCQA_Verify(TXID, XCQA_Sign(TXID, SK_reg), reg_epoch_pk) == true
    AND BLAKE3(TXID || xcqa_nonce)[0..REG_POW_LEADING_ZEROS] == 0x00...
```

`REG_POW_LAYERS` SHALL be configurable at genesis. Recommended: 4 layers (significantly easier than mining).

#### 6.3.3 Registration Transaction Wire Format

```rust
pub struct RegistrationTx {
    pub txid:           [u8; 32],
    pub uuid:           Uuid,
    pub pk_mldsa:       MlDsa65PublicKey,
    pub pk_xcqa:        XcqaPublicKey,
    pub timestamp:      u64,
    pub ref_block_hash: [u8; 64],       // BLAKE3-512
    pub pow_xcqa_sig:   XcqaSignature,  // PoW solution signature
    pub pow_nonce:      [u8; 32],       // for hash difficulty
    pub self_sig:       MlDsa65Sig,     // Sign(txid, sk_mldsa) - proves key ownership
}
```

### 6.4 Key Storage Recommendations

Implementors SHOULD store private keys encrypted with a user-supplied passphrase using Argon2id KDF. Private key files MUST NOT be committed to version control or transmitted over untrusted channels.

---

## 7. Consensus Layer (PoW)

### 7.1 PoW Puzzle Definition

For each block at height H with previous block header hash `prev_hash`:

```
epoch_pk_dict = generate_epoch_pk(BLAKE3-512(prev_header), H, difficulty_tier)

Miner must find (SK_epoch, xcqa_nonce) such that:
    (1) XCQA_Verify(block_candidate_header, sig, epoch_pk_dict) == true
        where sig = XCQA_Sign(block_candidate_header, SK_epoch)
    (2) BLAKE3(block_candidate_header || xcqa_nonce)[0..k] == 0x00...
        where k = fine_difficulty_leading_zeros
```

`SK_epoch` is discarded immediately after block broadcast. It MUST NOT be reused.

### 7.2 Difficulty Adjustment

Difficulty is controlled by two independent parameters:

#### 7.2.1 Coarse Difficulty: XCQA Layer Count Tier

| Tier | Layer Count | Relative Search Space |
|------|------------|----------------------|
| 1    | 8          | baseline             |
| 2    | 9          | ~exponential ×N      |
| 3    | 10         | ~exponential ×N²     |
| ...  | ...        | ...                  |

Layer count changes MUST be agreed by supermajority consensus (≥ 2/3 active validators by stake or block count) and take effect at the next epoch boundary.

#### 7.2.2 Fine Difficulty: Hash Leading Zeros

`fine_difficulty_leading_zeros` (k) is adjusted every `DIFFICULTY_WINDOW = 2016` blocks:

```
target_block_time = GENESIS_BLOCK_TIME_MS  // e.g., 60_000 ms
actual_time = timestamp[H] - timestamp[H - DIFFICULTY_WINDOW]
expected_time = target_block_time * DIFFICULTY_WINDOW

if actual_time < expected_time * 0.9:  k += 1   // too fast, increase difficulty
if actual_time > expected_time * 1.1:  k -= 1   // too slow, decrease difficulty
k = clamp(k, MIN_LEADING_ZEROS, MAX_LEADING_ZEROS)
```

#### 7.2.3 Oscillation Prevention

To prevent tier oscillation when switching coarse difficulty:
- Tier upgrades require 10 consecutive windows above threshold.
- Tier downgrades require 5 consecutive windows below threshold.
- Fine difficulty `k` is reset to `DEFAULT_FINE_ZEROS` on any tier change.

### 7.3 Block Candidate Construction

```rust
pub struct BlockCandidate {
    pub version:         u32,
    pub height:          u64,
    pub prev_hash:       [u8; 64],       // BLAKE3-512 of prev header
    pub merkle_root:     [u8; 64],       // BLAKE3-512 Merkle root of txs
    pub timestamp:       u64,            // Unix milliseconds
    pub difficulty_tier: u8,
    pub fine_difficulty: u8,             // k, leading zero count
    pub epoch_pk_hash:   [u8; 32],       // SHA256 of epoch_pk_dict (for verification)
}
```

Miners hash `BlockCandidate` (serialized via canonical little-endian encoding) as the PoW target message.

### 7.4 Block Finalization

```rust
pub struct FinalizedBlock {
    pub candidate:       BlockCandidate,
    pub miner_uuid:      Uuid,
    pub xcqa_sig:        XcqaSignature,  // Sign(candidate_bytes, SK_epoch)
    pub xcqa_nonce:      [u8; 32],       // satisfies hash difficulty
    pub transactions:    Vec<Transaction>,
    pub registrations:   Vec<RegistrationTx>,
}
```

Validation rules:

1. `prev_hash` MUST match BLAKE3-512 of the most recent finalized block header.
2. `epoch_pk_hash` MUST match SHA256 of the locally reconstructed `epoch_pk_dict`.
3. `XCQA_Verify(candidate_bytes, xcqa_sig, epoch_pk_dict)` MUST be true.
4. `BLAKE3(candidate_bytes || xcqa_nonce)[0..fine_difficulty]` MUST be all zero.
5. `merkle_root` MUST match Merkle root of all included transactions.
6. All included transactions MUST be individually valid (see Section 8).
7. `timestamp` MUST be within `[median_of_last_11_blocks, now + 7200_000]` ms.
8. `miner_uuid` MUST reference a registered account.

### 7.5 Chain Selection Rule

Longest cumulative work chain. Work per block:

```
work = 2^fine_difficulty × search_space(difficulty_tier)
```

---

## 8. Block and Transaction Structure

### 8.1 Transaction Types

```rust
pub enum TxType {
    Transfer,
    PrivacyPoolDeposit,
    PrivacyPoolWithdraw,
    AuthorizedDebit,       // "scan-to-pay" / contract deduction
    ContractDeploy,
    ContractCall,
    ComputeTaskSubmit,
    ComputeTaskSettle,
    StorageStore,
    StorageRetrieve,
    RegistrationPoW,       // handled separately in block
}
```

### 8.2 TXID Computation

All transactions (except registration) use:

```
tx_payload = tx_type_le8 || from_uuid || to_uuid || amount_comm_bytes
             || fee_comm_bytes || nonce_le64 || timestamp_le64
             || type_specific_fields
TXID = SHA512(tx_payload || from_uuid_bytes)
```

Mixing `from_uuid` into the hash prevents TXID malleability across accounts.

### 8.3 Transfer Transaction

```rust
pub struct TransferTx {
    pub txid:          [u8; 64],            // SHA-512
    pub from:          Uuid,
    pub to:            Uuid,
    pub amount_comm:   LatticeCommitment,   // Commit(amount, r_amount)
    pub fee_comm:      LatticeCommitment,   // Commit(fee, r_fee)
    pub remain_comm:   LatticeCommitment,   // Commit(balance - amount - fee, r_remain)
    pub range_proof:   LatticeRangeProof,   // aggregated; proves amount, fee, remain ∈ [0, 2^64)
    pub balance_proof: BalanceProof,        // proves remain = prev_balance - amount - fee
    pub nonce:         u64,
    pub timestamp:     u64,
    pub sender_sig:    MlDsa65Sig,
}
```

#### 8.3.1 Balance Proof

```
balance_proof proves (non-interactively via Fiat-Shamir):
    remain_comm = prev_balance_comm - amount_comm - fee_comm

i.e., the Pedersen commitments satisfy:
    remain_comm + amount_comm + fee_comm == prev_balance_comm

Prover knows: r_remain, r_amount, r_fee, r_prev such that:
    r_remain + r_amount + r_fee == r_prev  (mod group order)
```

Verification is O(1): one multi-scalar multiplication check.

### 8.4 Authorized Debit Transaction

Enables "scan-to-pay" payments and contract/compute fee deductions without the payer being online at execution time.

```rust
pub struct AuthorizedDebit {
    pub authorization_id: [u8; 64],          // SHA-512 unique ID
    pub payer:            Uuid,
    pub payee:            Uuid,
    pub max_amount_comm:  LatticeCommitment,  // Commit(max_amount, r)
    pub expiry:           u64,
    pub merchant_lock:    Option<Uuid>,
    pub payer_sig:        MlDsa65Sig,
}

pub struct AuthorizedDebitExecution {
    pub txid:               [u8; 64],
    pub authorization_id:   [u8; 64],
    pub actual_amount_comm: LatticeCommitment,
    pub amount_leq_proof:   LatticeAmountLeqProof,
    pub remaining_auth_comm: LatticeCommitment,
    pub executor_sig:       MlDsa65Sig,
}
```

`AmountLeqProof` proves `actual ≤ max` in Pedersen commitment space:
```
actual_amount_comm + diff_comm == max_amount_comm
range_proof(diff_comm) ∈ [0, 2^64)   // diff ≥ 0 ⟹ actual ≤ max
```

Authorized debits skip the balance non-negativity constraint for the debit amount itself; however, the payer's remaining balance MUST still satisfy the range proof.

---

## 9. Account Model and State

### 9.1 State Trie

Global state is stored in a BLAKE3-keyed Merkle Patricia Trie mapping:

```
UUID → AccountState
```

```rust
pub struct AccountState {
    pub pk_mldsa_hash:  [u8; 64],       // SHA-512(pk_mldsa) stored on-chain
    pub pk_xcqa_hash:   [u8; 64],       // SHA-512(pk_xcqa) stored on-chain
    pub balance_comm:   LatticeCommitment,
    pub nonce:          u64,
    pub reputation:     i64,
    pub code_hash:      Option<[u8; 64]>,  // SHA-512
    pub storage_root:   Option<[u8; 64]>,  // BLAKE3-512 of contract storage trie
    pub registered_at:  u64,
}
```

Full public keys are stored in a separate append-only key registry trie to avoid bloating the state trie. Transactions reference keys by their hash; validators resolve the full key when needed.

### 9.2 Nonce Rules

- Account nonce starts at 0 at registration.
- Each transaction from account `A` MUST include `nonce = current_nonce(A)`.
- On successful inclusion, nonce increments by 1.
- Transactions with incorrect nonce are rejected and MUST NOT be included in a block.

### 9.3 Balance Invariant

At every block boundary, for all accounts:

```
∀ account A: balance_comm(A) encodes a value v ≥ 0
```

This is enforced by range proofs in every transfer and contract operation. Nodes MUST reject blocks that would violate this invariant.

---

## 10. Smart Contract Layer (EVM)

### 10.1 EVM Variant

XCQA-Chain uses a modified EVM with the following changes from Ethereum Yellow Paper:

- Hash opcode: `KECCAK256` replaced with `BLAKE3_256` (same gas cost).
- Native Pedersen commitment precompile at address `0x09`.
- Native ML-DSA-65 verify precompile at address `0x0A`.
- Native XCQA verify precompile at address `0x0B`.
- Gas token: XCQA-Chain native token (balances via Pedersen commitments; gas is paid separately as a cleartext u64 per transaction for simplicity).
- No `SELFDESTRUCT` opcode (removed).
- Account addressing: UUIDv7 bytes (16 bytes) used as EVM address in a 32-byte zero-padded word.

### 10.2 Fee Model

```
total_fee = base_fee + compute_fee + storage_fee + inspector_fee

base_fee: fixed protocol minimum (set at genesis)
compute_fee: Firecracker/QEMU CPU core-seconds × CORE_RATE
storage_fee: DHT MB × DURATION_EPOCHS × STORAGE_RATE
inspector_fee: fixed fraction of compute_fee (e.g., 10%)
```

All fees are denominated in the native token and expressed as cleartext u64 values for gas accounting. The EVM pre-deducts the declared maximum fee at transaction start and refunds unused amounts at completion.

### 10.3 Contract Authorized Debit Integration

Contracts MAY deduct from payer balances using a stored `AuthorizedDebit` object. The EVM precompile at `0x0C` validates:

1. Authorization signature is valid ML-DSA-65.
2. Expiry has not passed.
3. Actual debit ≤ max_amount (via `AmountLeqProof`).
4. Merchant lock matches contract address (if set).

---

## 11. Compute Layer (Firecracker / QEMU)

### 11.1 Microservice Package Format

```
microservice.tar.zst
├── rootfs.ext4       (squashfs or ext4 image, max 2 GB)
├── manifest.toml
└── keyring.seal      (sealed keyring, see §11.5)
```

#### 11.1.1 Manifest Format

```toml
[microservice]
name    = "example-service"
version = "1.0.0"
image_hash = "blake3_512_hex_of_rootfs"   # integrity verification

[resources]
vcpus      = 2         # maximum vCPUs
memory_mb  = 512       # maximum RAM in MiB
disk_mb    = 1024      # maximum ephemeral disk
network    = true      # enable network interface

[limits]
max_cpu_core_seconds = 300
max_mem_mb_seconds   = 153_600   # 512 MB × 300 s
max_net_mb           = 100

[entrypoint]
kernel  = "vmlinux"   # minimal kernel inside rootfs
cmdline = "console=ttyS0 reboot=k panic=1 pci=off"
init    = "/sbin/init-microservice"
```

### 11.2 Task Routing

Task routing uses consistent hashing:

```
vnode_id = BLAKE3(TXID)[0..8] as u64 % VNODE_RING_SIZE
node_id  = ring.lookup(vnode_id)    // Kademlia DHT lookup
```

The executing node is uniquely determined by TXID. No coordinator election is needed.

### 11.3 Execution Protocol

```
State machine per compute task:

SUBMITTED → ASSIGNED → RUNNING → COMPLETED
                   ↓              ↓
               TIMEOUT         SETTLED (on-chain)
                   ↓
               REASSIGNED (next vnode in ring)
               + reputation_penalty(original_node)
```

Timeout period: `COMPUTE_TIMEOUT_MS` (configurable, default 30_000 ms).

On timeout, the task is reassigned to the next vNode in the ring. The original node's reputation decreases by `TIMEOUT_REPUTATION_PENALTY` (default -10).

### 11.4 Inspector Mechanism

For every compute task, an Inspector is assigned in parallel:

```
inspector_vnode = BLAKE3(TXID || b"INSPECTOR")[0..8] as u64 % VNODE_RING_SIZE
```

The inspector vNode MUST NOT be the same as the executor vNode. If collision occurs, use next distinct vNode.

The Inspector:

1. Connects to the executor's microVM monitor endpoint (virtio serial or vsock).
2. Samples resource usage at `INSPECTOR_SAMPLE_INTERVAL_MS = 500` ms intervals.
3. Records CPU time, peak memory, and network bytes.
4. At task completion, computes actual resource totals.
5. Signs the resource report with its ML-DSA-65 key.
6. Submits the `ComputeTaskSettle` transaction to the EVM layer.

Multiple Inspectors MAY be assigned for high-value tasks (configurable in manifest). Final resource figures are the median of all Inspector reports.

### 11.5 Keyring

The keyring is sealed inside `keyring.seal` in the microservice package:

- Keyring keys are generated at service build time and sealed using a service-specific master key.
- The master key is held by the service operator and never transmitted to executing nodes.
- Executing nodes can access only what the service's `init` process decrypts internally.
- Inspector nodes observe only resource counters, NOT memory contents or decrypted data.

External nodes holding encrypted DHT data can never decrypt it without the keyring. Data privacy is guaranteed by the rootfs boundary.

### 11.6 Platform: Linux (Firecracker)

```rust
// Firecracker VMM invocation
let vm_config = VmConfig {
    vcpu_count: manifest.resources.vcpus,
    mem_size_mib: manifest.resources.memory_mb,
    ..Default::default()
};
let boot_source = BootSource {
    kernel_image_path: rootfs_mount.kernel_path(),
    boot_args: manifest.entrypoint.cmdline.clone(),
};
// attach rootfs as virtio-blk drive
// attach vsock for inspector channel
```

Firecracker MUST be invoked with `--no-api` flag in production to disable the management API after configuration.

### 11.7 Platform: Windows (QEMU MicroVM)

On Windows hosts, Firecracker is unavailable. QEMU SHALL be used with the `microvm` machine type:

```
qemu-system-x86_64 \
    -M microvm,x-option-roms=off,pit=off,pic=off,rtc=off \
    -enable-kvm (if available; else -accel whpx or -accel tcg) \
    -cpu host \
    -smp {vcpus} \
    -m {memory_mb}M \
    -kernel {kernel_path} \
    -append "{cmdline}" \
    -drive id=rootfs,file={rootfs_path},format=raw,if=none \
    -device virtio-blk-device,drive=rootfs \
    -device virtio-serial-device \
    -chardev socket,path={vsock_path},id=inspector \
    -device virtconsole,chardev=inspector \
    -nographic -nodefaults -no-reboot
```

The Rust VMM abstraction layer SHALL expose a unified `MicroVm` trait implemented by both `FirecrackerVm` and `QemuMicroVm`.

```rust
pub trait MicroVm: Send + Sync {
    fn start(&mut self, manifest: &Manifest, rootfs: &Path) -> Result<()>;
    fn wait_completion(&mut self, timeout: Duration) -> Result<TaskResult>;
    fn inspector_channel(&self) -> Box<dyn Read + Write + Send>;
    fn kill(&mut self) -> Result<()>;
}

#[cfg(target_os = "linux")]
pub type PlatformVm = FirecrackerVm;

#[cfg(target_os = "windows")]
pub type PlatformVm = QemuMicroVm;
```

---

## 12. Storage Layer (DHT / RAID6)

### 12.1 DHT Design

Based on Kademlia with BLAKE3-256 node IDs (32 bytes).

```
node_id = BLAKE3(uuid_bytes || pk_mldsa_hash)  // deterministic, Sybil-resistant

XOR metric: distance(a, b) = a XOR b
k-bucket size: K = 20
alpha (parallelism): 3
```

Node IDs are derived from registered account identities, preventing Sybil nodes from flooding the ring without paying the registration PoW cost.

### 12.2 RAID6 Erasure Coding

Data is split into `N_DATA` shards with `N_PARITY = 2` Reed-Solomon parity shards.

Recommended configuration:
- `N_DATA = 6`, `N_PARITY = 2` (total 8 shards)
- Maximum tolerable shard loss: 2
- Each shard stored on a distinct DHT node

```rust
pub struct StorageManifest {
    pub content_hash:   [u8; 64],       // BLAKE3-512 of plaintext
    pub shard_hashes:   Vec<[u8; 32]>,  // SHA256 of each encrypted shard
    pub n_data:         u8,
    pub n_parity:       u8,
    pub encrypted:      bool,
    pub keyring_id:     Option<[u8; 32]>, // which keyring seals this data
    pub chunk_size:     u32,
}
```

### 12.3 Write Protocol (Quorum)

Write is confirmed only when `n_data + 1` shard nodes acknowledge storage (more than parity shards, ensuring recovery is possible even if acks are lost):

```
quorum_required = n_data + 1   // e.g., 7 of 8 for N_DATA=6, N_PARITY=2
```

Write procedure:

1. Encrypt data with keyring-derived key (AES-256-GCM or ChaCha20-Poly1305).
2. Apply RAID6 encoding → 8 shards.
3. Compute target DHT node for each shard: `node = dht.lookup(shard_hash)`.
4. Send shards to target nodes in parallel.
5. Collect acknowledgements; proceed when `quorum_required` received within timeout.
6. Submit `StorageStore` transaction to EVM with `StorageManifest`.

### 12.4 Read Protocol

1. Look up `StorageManifest` from EVM state by content hash.
2. Contact shard nodes in parallel; any `n_data` responsive shards suffice.
3. Reconstruct via RAID6 decoding.
4. Decrypt with keyring.
5. Verify `BLAKE3-512(plaintext) == content_hash`.

### 12.5 Storage Billing

Storage fees are assessed per epoch (epoch = `STORAGE_EPOCH_BLOCKS` blocks):

```
fee_per_epoch = ceil(plaintext_bytes / 1_048_576) × STORAGE_RATE_PER_MB_EPOCH
```

If an account's storage fee authorization expires or the payer's balance is insufficient, shard nodes MAY delete the data after `STORAGE_GRACE_EPOCHS` grace epochs.

---

## 13. Privacy Transfer Protocol

### 13.1 Overview

The privacy transfer protocol enables transfers where sender and receiver cannot be linked on-chain. Sender identity is visible at deposit time (normal transfer); receiver identity is visible at withdrawal time. However, the deposit and withdrawal events are cryptographically unlinkable.

### 13.2 Privacy Pool State

The privacy pool is a set of entries stored in a Merkle tree maintained by the EVM:

```rust
pub struct PoolEntry {
    pub commitment:     XcqaCommitment,     // XCQA_Encrypt(nonce, pk_once)
    pub amount_comm:    LatticeCommitment,  // Commit(amount, r_amount)
    pub note:           XcqaCiphertext,     // XCQA_Encrypt(amount||r||nonce||pk_once, receiver_pk_xcqa)
    pub nullifier_hash: [u8; 64],           // BLAKE3-512(nullifier)
    pub deposited_at:   BlockHeight,
    pub pool_epoch:     u32,                // which pool epoch this entry belongs to
}
```

The pool Merkle tree root is updated on every deposit and withdrawal.

### 13.3 Deposit Protocol

```
Sender actions:
    1. Generate one-time keypair: (pk_once, sk_once) ← XCQA_KeyGen()
    2. Compute:
        nonce      = random_bytes(XCQA_NONCE_LEN)
        commitment = XCQA_Encrypt(nonce, pk_once)
        r_amount   = random_scalar()
        amount_comm = Commit(amount, r_amount)
        note       = XCQA_Encrypt(
                        amount_bytes || r_amount_bytes || nonce || pk_once_bytes,
                        receiver_pk_xcqa
                     )
    3. Submit PrivacyPoolDeposit transaction:
        - Deducts amount + fee from sender balance (visible, with range proof)
        - Adds PoolEntry to pool Merkle tree
    4. Transmit note delivery via any channel to receiver
       (note delivery is not on-chain; receiver can also scan all pool entries)
```

Note: `pk_once` is one-time. It is never associated with any registered account. `sk_once` is discarded after deposit. The sender possesses no information allowing them to later identify the receiver.

### 13.4 Withdrawal ZK Proof

Receiver withdraws by producing a non-interactive ZK proof.

#### 13.4.1 Proof Statement

```
Public inputs:
    merkle_root         : [u8; 64]              // BLAKE3-512 pool tree root
    nullifier           : [u8; 64]              // BLAKE3-512(sk_xcqa || commitment)
    withdraw_comm       : LatticeCommitment     // Commit(withdraw_amount, r_w)
    remain_comm         : LatticeCommitment     // Commit(amount - withdrawn - withdraw_amount, r_r)
    receiver_balance_delta: LatticeCommitment   // credits to receiver

Private inputs (witness):
    sk_xcqa             : XcqaPrivateKey
    commitment          : XcqaCommitment
    pk_once             : XcqaPublicKey
    nonce               : Bytes
    amount              : u64
    r_amount            : LatticeRandVec
    withdrawn_so_far    : u64
    r_withdrawn         : LatticeRandVec
    withdraw_amount     : u64
    r_w                 : LatticeRandVec
    r_r                 : LatticeRandVec
    merkle_path         : Vec<([u8;64], Side)>
```

#### 13.4.2 Proof Constraints

The ZK proof must satisfy all of the following constraints:

**Constraint 1 — Decryption correctness:**
```
XCQA_Decrypt(note, sk_xcqa) == amount_bytes || r_amount_bytes || nonce || pk_once_bytes
```

**Constraint 2 — Commitment reconstruction:**
```
XCQA_Encrypt(nonce, pk_once) == commitment
```

**Constraint 3 — Pedersen binding:**
```
Commit(amount, r_amount) == amount_comm   (the stored pool entry amount_comm)
```

**Constraint 4 — Withdrawal arithmetic (EC subtraction):**
```
withdraw_comm + remain_comm == amount_comm - Commit(withdrawn_so_far, r_withdrawn)

Equivalently:
Commit(withdraw_amount, r_w) + Commit(amount - withdrawn_so_far - withdraw_amount, r_r)
    == Commit(amount - withdrawn_so_far, r_amount - r_withdrawn)
```

**Constraint 5 — Nullifier correctness:**
```
BLAKE3-512(sk_xcqa_bytes || commitment_bytes) == nullifier
```

**Constraint 6 — Merkle membership:**
```
merkle_verify(commitment, merkle_path, merkle_root) == true
```

**Constraint 7 — Range proofs (Bulletproofs):**
```
withdraw_amount ∈ [0, 2^64)
amount - withdrawn_so_far - withdraw_amount ∈ [0, 2^64)
```

#### 13.4.3 Non-Interactive Proof via Fiat-Shamir

All challenges in the proof are derived as:

```
challenge_i = BLAKE3(
    transcript_so_far ||
    public_inputs_serialized ||
    latest_finalized_block_hash
)
```

Including `latest_finalized_block_hash` binds the proof to a point in time, preventing proof replay across chain forks.

#### 13.4.4 Partial Withdrawal

A single pool entry MAY be withdrawn in multiple partial transactions. Each withdrawal:

1. Produces a new `remain_comm` encoding the remaining balance.
2. Emits a unique nullifier per withdrawal: `BLAKE3-512(sk_xcqa || commitment || withdrawal_index_le64)`
3. Does NOT reveal which pool entry is being partially consumed.

### 13.5 Double-Spend Prevention

The EVM maintains a set of used nullifiers:

```rust
pub type NullifierSet = BTreeSet<[u8; 64]>;  // BLAKE3-512 nullifiers
```

On withdrawal:
1. Verify `nullifier` is not in `NullifierSet`.
2. Accept withdrawal and add `nullifier` to `NullifierSet`.
3. Nullifiers are permanent and MUST NOT be pruned.

A `nullifier` leaks no information about `sk_xcqa`, `commitment`, or `amount` (BLAKE3 preimage resistance).

### 13.6 XCQA Circuit Complexity Advantage

XCQA operations translate to arithmetic constraints as follows:

| XCQA Operation | Circuit Constraint Type | Approximate Gate Count |
|---|---|---|
| Dictionary lookup (per entry) | Table lookup constraint | O(dict_size) |
| Bit overlay (per layer) | Range + bitwise constraint | O(output_bits) |
| XOR (challenge response) | Linear constraint | O(1) per bit |
| Layer sequencing | Copy constraint chain | O(n_layers) |
| Lattice commitment open | Module-SIS linear relation | O(k × n) |
| **Total XCQA + Lattice** | | **~O(n_layers × dict_size + k×n)** |
| ML-DSA-65 verify (reference) | NTT polynomial multiply | **~O(2^20) gates** |

XCQA circuit complexity is estimated 2–3 orders of magnitude smaller than ML-DSA-65, making proof generation feasible on consumer hardware.

### 13.7 Privacy Guarantees

| Property | Guarantee |
|---|---|
| Sender identity | Visible at deposit (unavoidable without ring signatures) |
| Receiver identity | Visible at withdrawal (account receives funds) |
| Sender-receiver linkage | Cryptographically unlinkable |
| Transfer amount | Hidden (Pedersen + Bulletproofs) |
| Partial withdrawal linkage | Unlinkable across withdrawals |
| Double-spend | Prevented by nullifier set |

---

## 14. Network Protocol

### 14.1 Transport

All peer-to-peer communication SHALL use TCP with TLS 1.3. Node identity in TLS is bound to ML-DSA-65 certificates.

### 14.2 Message Types

```rust
pub enum Message {
    // Handshake
    Handshake       { version: u32, node_uuid: Uuid, node_pk_mldsa_hash: [u8;32] },
    HandshakeAck    { accepted: bool, peer_list: Vec<SocketAddr> },

    // Peer discovery
    GetPeers        { max: u16 },
    PeerList        { peers: Vec<(Uuid, SocketAddr)> },

    // Block sync
    GetBlocks       { start_height: u64, max_count: u16 },
    BlockData       { block: FinalizedBlock },
    BlockHeader     { header: BlockCandidate, height: u64 },

    // Mempool
    NewTransaction  { tx: Transaction },
    NewBlock        { block: FinalizedBlock },

    // Registration
    NewRegistration { reg: RegistrationTx },

    // Compute
    ComputeTaskSubmit   { task: ComputeTask },
    ComputeTaskResult   { txid: [u8;32], result_hash: [u8;32], sig: MlDsa65Sig },
    InspectorReport     { txid: [u8;32], report: ResourceReport, sig: MlDsa65Sig },

    // Storage
    ShardStore      { manifest_hash: [u8;32], shard_index: u8, data: Vec<u8> },
    ShardAck        { manifest_hash: [u8;32], shard_index: u8 },
    ShardFetch      { manifest_hash: [u8;32], shard_index: u8 },
    ShardData       { manifest_hash: [u8;32], shard_index: u8, data: Vec<u8> },

    // Privacy pool
    PoolSync        { entries_since: BlockHeight },
    PoolEntries     { entries: Vec<PoolEntry> },
}
```

All messages are serialized using `bincode` (little-endian, fixed-size integers) and prefixed with a 4-byte little-endian length field.

### 14.3 Bootstrap

New nodes connect to one or more hard-coded or user-configured bootstrap nodes. Bootstrap nodes provide:

1. Initial peer list.
2. Block headers for chain tip discovery.
3. Full block sync via `GetBlocks`.

Nodes SHOULD maintain connections to at least `MIN_PEERS = 8` and at most `MAX_PEERS = 125` peers.

---

## 15. Fee and Incentive Model

### 15.1 Fee Components

| Component | Recipient | Rate |
|---|---|---|
| Base transfer fee | Miner | `BASE_FEE` (genesis constant) |
| EVM gas | Miner | `gas_used × gas_price` |
| Compute CPU fee | Executor node | `core_seconds × CORE_RATE` |
| Compute memory fee | Executor node | `mb_seconds × MEM_RATE` |
| Compute network fee | Executor node | `net_mb × NET_RATE` |
| Inspector fee | Inspector node | `INSPECTOR_FRACTION × compute_fee` |
| Storage fee | Storage nodes | `mb × epochs × STORAGE_RATE` |

### 15.2 Miner Reward

```
miner_reward = Σ(base_fees) + Σ(evm_gas_fees) + block_subsidy

block_subsidy = INITIAL_SUBSIDY × (1/2)^(height / HALVING_INTERVAL)
```

`HALVING_INTERVAL` is set at genesis (e.g., 210_000 blocks).

### 15.3 Fee Settlement Flow

```
Transaction submitted → EVM pre-deducts max_fee from sender
                      ↓
Task executes         → Inspector measures actual usage
                      ↓
ComputeTaskSettle     → EVM releases:
                           actual_fee → executor
                           inspector_fee → inspector
                           refund → sender
                      ↓
Block finalized       → miner collects base_fee + gas_fees
```

---

## 16. Security Considerations

### 16.1 XCQA PoW Security

The security of the PoW mechanism depends on XCQA key recovery hardness. This is an experimental cryptosystem without formal reduction to a standard hard problem. Implementors SHOULD monitor cryptanalysis literature and be prepared to increase layer count or migrate to a different PoW mechanism if weaknesses are discovered.

The fine difficulty (hash leading zeros) provides a fallback security layer that remains secure even if XCQA is weakened, as long as BLAKE3 preimage resistance holds.

### 16.2 ZK Proof Soundness

ZK proofs in the privacy protocol are non-interactive via Fiat-Shamir. Soundness relies on BLAKE3 behaving as a random oracle. No trusted setup is required.

The binding `latest_finalized_block_hash` in challenge derivation prevents proof replay across forks. Implementors MUST enforce a maximum proof age of `ZK_PROOF_MAX_AGE_BLOCKS = 100` blocks.

### 16.3 Quantum Resistance

| Component | Mechanism | Quantum Status |
|---|---|---|
| Account signatures | ML-DSA-65 (FIPS 204) | Quantum-resistant |
| XCQA ZK witness | XCQA (experimental) | Conjectured resistant |
| Hash functions | BLAKE3-512, SHA-512 | Grover halves to 256-bit effective; acceptable |
| Commitments | Module-SIS (ML-DSA-65 lattice) | Quantum-resistant |
| Range proofs | BDLOP + Σ-Protocol | Quantum-resistant |

### 16.4 Sybil Resistance

Registration requires PoW, binding new identities to real computational cost. DHT node IDs are derived from registered account identities. An attacker wishing to flood the DHT must pay registration PoW costs proportional to the number of fake identities.

### 16.5 Inspector Collusion

If executor and inspector are the same physical operator (but different vNodes), they may collude to misreport resource usage. Mitigations:

- For high-value tasks, require `N_INSPECTORS ≥ 3` with median aggregation.
- Reputation slashing for nodes found reporting inconsistently across tasks.
- Random audits: a fraction of tasks are re-executed by a third node and results compared.

---

## 17. Rust Implementation Guidelines

### 17.1 Workspace Structure

```
xcqa-chain/
├── Cargo.toml                  (workspace)
├── crates/
│   ├── xcqa-crypto/            (XCQA, ML-DSA-65, Pedersen, Bulletproofs)
│   ├── xcqa-chain-core/        (block, tx, state trie, consensus rules)
│   ├── xcqa-chain-evm/         (EVM engine + precompiles)
│   ├── xcqa-chain-pow/         (PoW solver, difficulty adjustment)
│   ├── xcqa-chain-privacy/     (ZK proof generation and verification)
│   ├── xcqa-chain-compute/     (MicroVM abstraction, Firecracker, QEMU)
│   ├── xcqa-chain-storage/     (DHT, RAID6, keyring)
│   ├── xcqa-chain-net/         (P2P, TLS, message codec)
│   └── xcqa-chain-node/        (full node binary, CLI)
```

### 17.2 Key Crate Dependencies

```toml
[workspace.dependencies]
# Cryptography
blake3        = "1"
sha2          = "0.10"          # SHA-512
rand          = "0.8"
rand_core     = "0.6"
zeroize       = "1"
subtle        = "2"             # constant-time comparisons
ml-dsa        = "0.2"          # ML-DSA-65 (FIPS 204)
# Note: curve25519-dalek and bulletproofs removed (replaced by lattice)
# Lattice commitments and range proofs: implemented in xcqa-crypto crate
# using ml-dsa ring arithmetic primitives

# Serialization
bincode       = "2"
serde         = { version = "1", features = ["derive"] }

# Compression
zstd          = "0.13"

# Async runtime
tokio         = { version = "1", features = ["full"] }

# DHT / Networking
libp2p        = "0.53"

# EVM
revm          = "8"

# Erasure coding
reed-solomon-erasure = "6"

# UUID
uuid          = { version = "1", features = ["v7"] }

# Error handling
thiserror     = "1"
anyhow        = "1"
```

### 17.3 Platform Feature Flags

```toml
[features]
default      = []
firecracker  = ["dep:firecracker-sdk"]   # Linux only
qemu-backend = ["dep:qemu-process"]      # Windows fallback
```

Firecracker dependency SHALL be wrapped:

```rust
#[cfg(feature = "firecracker")]
mod firecracker_vm;

#[cfg(feature = "qemu-backend")]
mod qemu_vm;
```

### 17.4 Critical Safety Requirements

- All private key material MUST be stored in `zeroize`-on-drop wrappers.
- XCQA private keys used for PoW MUST be zeroed from memory immediately after block broadcast.
- All cryptographic random number generation MUST use `rand::CryptoRng` trait implementations.
- Constant-time comparisons MUST be used for all security-sensitive byte comparisons (`subtle` crate).
- No `unsafe` blocks in `xcqa-crypto` crate without documented justification and audit.

### 17.5 Serialization Canonical Form

All data structures that are hashed or signed MUST use a canonical serialization:

- Integer fields: little-endian fixed-width.
- Variable-length fields: prefixed with 4-byte little-endian length.
- Enum variants: 1-byte discriminant.
- Optional fields: 1-byte presence flag followed by value if present.

The canonical form MUST be independent of the host platform's endianness.

---

### 12.6 Zstd Compression Policy

All heavy data SHOULD be compressed with zstd before storage or transmission to reduce chain and network overhead.

#### 12.6.1 Compression Targets and Levels

| Data Type | Zstd Level | Applied At | Expected Ratio |
|---|---|---|---|
| Block bodies (transactions) | 3 | DHT storage + wire sync | 2–4× |
| Range proofs (lattice) | 6 | Transaction payload | 3–5× |
| Microservice rootfs shards | 3 | DHT storage | varies |
| Archive snapshots | 19 | Chain compression (§18) | 5–10× |
| P2P block sync messages | 1 | Wire protocol (low latency) | 2–3× |

Compression level 1 is used for real-time wire protocol to minimize latency. Higher levels are used for persistent storage where throughput matters more than latency.

#### 12.6.2 Framing

Compressed payloads SHALL be prefixed with a 1-byte flag indicating compression:

```
0x00  uncompressed
0x01  zstd compressed
```

Followed by 4-byte little-endian uncompressed length (for pre-allocation), then compressed bytes.

#### 12.6.3 Mandatory Compression

The following data MUST be compressed before DHT storage:

- Block bodies exceeding 4096 bytes
- All lattice range proofs
- All microservice rootfs images

Block headers MUST NOT be compressed (they are small and frequently accessed for chain validation).

---

## 18. Chain Compression and Historical Archival Protocol

### 18.1 Overview

As the chain grows, full-node storage requirements increase without bound. XCQA-Chain supports periodic archival of historical chain data into a compressed snapshot database, replacing raw block history with a verified state snapshot. Full historical data is preserved by opt-in audit nodes.

### 18.2 Archival Trigger Conditions

Archival is triggered when ALL of the following conditions are simultaneously true:

```
(1) raw_chain_size_on_disk > ARCHIVE_SIZE_THRESHOLD    // e.g., 50 GB
(2) current_pool_epoch > archive_candidate_epoch        // pool has been forked
(3) old_pool_epoch.entry_count == 0                    // old pool fully drained
(4) height > archive_candidate_height + ARCHIVE_MIN_AGE // e.g., 1000 blocks of margin
```

Condition (3) is the critical gate: the old privacy pool must be completely empty before its epoch's chain history can be archived. This ensures no outstanding ZK proof references chain state that would be discarded.

### 18.3 Privacy Pool Epoch Fork

When condition (1) is first met and condition (3) is not yet true, the network enters **pool migration mode**:

```
PoolMigrationAnnouncement:
    old_epoch:      u32
    new_epoch:      u32          // old_epoch + 1
    migration_start_block: u64
    drain_deadline_block:  u64   // migration_start_block + POOL_DRAIN_WINDOW
    announcement_sig: MlDsa65Sig // from proposing node
```

During `POOL_DRAIN_WINDOW` (recommended: 10_000 blocks ≈ ~7 days at 60s block time):

- New deposits MUST use `new_epoch` pool.
- Old pool (`old_epoch`) accepts withdrawals only.
- Existing entries in `old_epoch` pool remain valid and withdrawable.
- After `drain_deadline_block`, deposits to `old_epoch` are rejected by consensus.

### 18.4 Archive Database Format

The archive database (`archive_NNNN.db`) is a zstd-compressed binary file containing:

```rust
pub struct ArchiveDb {
    pub format_version:    u32,
    pub archive_id:        u32,              // sequential archive number
    pub genesis_hash:      [u8; 64],         // BLAKE3-512 of genesis block
    pub start_block_height: u64,
    pub start_block_hash:  [u8; 64],         // BLAKE3-512; new sync anchor
    pub archived_up_to_height: u64,
    pub archived_up_to_hash:   [u8; 64],
    pub created_at:        u64,              // Unix ms

    // Complete state snapshot at archived_up_to_height
    pub state_trie_root:   [u8; 64],
    pub accounts:          Vec<AccountState>,
    pub key_registry:      Vec<(Uuid, MlDsa65PublicKey, XcqaPublicKey)>,
    pub nullifier_set:     Vec<[u8; 64]>,    // ALL nullifiers ever used; MUST be complete
    pub contract_states:   Vec<(Uuid, ContractState)>,

    // Pool state
    pub active_pool_epoch: u32,
    pub pool_merkle_root:  [u8; 64],

    // Integrity
    pub state_hash:        [u8; 64],         // BLAKE3-512 of all above fields
    pub validator_sigs:    Vec<(Uuid, MlDsa65Sig)>,  // ≥ 2/3 of active validators
}
```

The archive is serialized with canonical little-endian encoding, then zstd-compressed at level 19.

### 18.5 Two-Phase Voting Protocol

Archival requires two independent 2/3 supermajority votes to prevent race conditions between database construction and confirmation.

#### 18.5.1 Phase 1: Archive Proposal

```rust
pub struct ArchiveProposal {
    pub archive_id:         u32,
    pub proposer_uuid:      Uuid,
    pub archive_db_hash:    [u8; 64],    // BLAKE3-512 of uncompressed archive_db
    pub archive_db_size:    u64,         // bytes, uncompressed
    pub start_block_height: u64,
    pub start_block_hash:   [u8; 64],
    pub archived_up_to:     u64,
    pub nullifier_count:    u64,         // for cross-checking
    pub proposal_block:     u64,
    pub proposer_sig:       MlDsa65Sig,
}
```

Phase 1 voting rules:

1. Proposer constructs `archive_db`, computes its BLAKE3-512 hash, and broadcasts `ArchiveProposal`.
2. Each validator independently reconstructs the archive state from local chain data and verifies `archive_db_hash` matches.
3. Validators that agree broadcast a Phase 1 vote: `Sign(ArchiveProposal fields, sk_mldsa)`.
4. Phase 1 passes when `≥ 2/3` of active validators (by count) have submitted valid votes.
5. Phase 1 votes are included in on-chain blocks as special transactions.
6. Phase 1 timeout: `ARCHIVE_VOTE_TIMEOUT_BLOCKS = 500` blocks. If timeout, proposal is abandoned.

#### 18.5.2 Phase 2: Archive Confirmation

```rust
pub struct ArchiveConfirmation {
    pub archive_id:          u32,
    pub phase1_vote_root:    [u8; 64],   // Merkle root of Phase 1 vote set
    pub confirmer_uuid:      Uuid,
    pub confirmer_sig:       MlDsa65Sig,
}
```

Phase 2 voting rules:

1. After Phase 1 passes, validators verify that Phase 1 votes are on-chain and the vote set is correct.
2. Each validator broadcasts a Phase 2 confirmation vote.
3. Phase 2 passes when `≥ 2/3` confirmations received.
4. Phase 2 confirms that validators have verified the Phase 1 vote set on-chain (prevents Phase 1 signature replay from an earlier failed proposal).
5. Phase 2 timeout: `ARCHIVE_VOTE_TIMEOUT_BLOCKS = 500` blocks.

#### 18.5.3 Archive Finalization

On Phase 2 passing:

1. The `ArchiveDb` with all Phase 1 + Phase 2 validator signatures is published to DHT storage.
2. Blocks `[genesis, archived_up_to_height]` may be deleted from full node storage.
3. New nodes syncing start from `start_block_height` using `start_block_hash` as trust anchor.
4. The nullifier set in the archive is the authoritative set; it MUST be loaded before processing any new privacy pool transactions.

### 18.6 New Node Sync with Archive

```
New node bootstrap:

1. Download latest ArchiveDb from DHT (verify BLAKE3-512 hash)
2. Verify ≥ 2/3 ML-DSA-65 signatures against registered validator keys
3. Load state trie, account balances, nullifier set from archive
4. Sync incremental blocks from start_block_height to chain tip
5. Resume normal operation
```

Archive integrity is guaranteed by the 2/3 validator signatures. A new node trusts the archive if and only if it trusts the validator set (same trust assumption as trusting the chain itself).

### 18.7 Audit Nodes

Nodes that retain full chain history are designated **audit nodes**. Audit nodes:

- Store all raw blocks from genesis, unarchived.
- Serve historical block requests from other nodes on demand.
- MAY charge a fee for historical data access (out of protocol scope).
- Are not required for normal chain operation.
- SHOULD exist in sufficient number for chain auditability.

Audit node status is self-declared and requires no special registration. Any full node that retains history qualifies.

### 18.8 Archive Frequency

```
Recommended ARCHIVE_SIZE_THRESHOLD: 50 GB
Minimum interval between archives: ARCHIVE_MIN_INTERVAL_BLOCKS = 100_000
```

Multiple archives accumulate sequentially. Each archive references the previous:

```
archive_N.start_block_hash == archive_(N-1).start_block_hash of its own anchor
```

New nodes MAY skip intermediate archives and sync from the latest, provided they load the cumulative nullifier set from all previous archives (included in each archive's nullifier set by construction).

---

*End of XCQA-Chain Technical Specification v0.2.0-draft*

*Document Number: XCQA-CHAIN-SPEC-2026-001*
*Status: Draft — subject to revision*
*Authors: rand0mdevel0per, Anthropic Claude Sonnet 4.6*
