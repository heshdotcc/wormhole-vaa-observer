# ADR-02: Domains

## Wormhole

This project inherits the **Wormhole VAA structure**, ensuring compatibility with cross-chain messaging.

VAAs are **binary-encoded**, signed messages used to verify and relay events between blockchain networks.

### **VAA Structure**

| **Field**               | **Size (bytes)** | **Type**        | **Description** |
|-------------------------|----------------|----------------|----------------|
| **Version**            | 1              | `u8`           | Protocol version (always `1`) |
| **Guardian Set Index** | 4              | `u32`          | Index of the Guardian set that signed this VAA |
| **Number of Signatures** | 1             | `u8`           | Number of Guardian signatures |
| **Signatures**         | `66 * N`       | `Vec<u8>`      | Guardian signatures (`N` = number of signatures) |
| **Timestamp**          | 4              | `u32`          | Unix timestamp (UTC) |
| **Nonce**             | 4              | `u32`          | Unique number per VAA |
| **Emitter Chain ID**   | 2              | `u16`          | Source blockchain ID |
| **Emitter Address**    | 32             | `[u8; 32]`     | Address of the emitter (on the source chain) |
| **Sequence Number**    | 8              | `u64`          | Monotonically increasing sequence number |
| **Consistency Level**  | 1              | `u8`           | Required consistency level |
| **Payload**           | Variable       | `Vec<u8>`      | Encoded transaction/event data |

> **Note:** The VAA is **Base64-encoded** but contains **binary data**, not a simple string.

### **Implementation**

A working VAA decoder implementation can be found in [backend vaa.rs logic](../microservices/backend/src/domain/wormhole/rest/vaa.rs) offered by the `/observer/vaas/decode` endpoint.

## **References**
- 📖 [Wormhole VAA Documentation](https://wormhole.com/docs/learn/infrastructure/vaas/)
- 🛠 [Solidity VAA Parsing (Solana)](https://github.com/wormhole-foundation/wormhole-solidity-sdk/blob/main/src/libraries/VaaLib.sol)
- 🦀 [Rust Wormhole VAA Parser](https://docs.rs/wormhole-vaas-serde)
- 🔧 [Wormhole Rust SDK](https://github.com/wormhole-foundation/wormhole/tree/main/sdk/rust) - Contains VAA parsing utilities
- 📚 [VAAs & Protocols Documentation](https://wormhole.com/docs/build/applications/wormhole-sdk/vaas-protocols/)

### **Signature Structure**

Each signature in the signatures array consists of:
- 1 byte for the guardian index
- 65 bytes for the signature data (recoverable ECDSA signature)

This gives us the total of 66 bytes per signature as shown in the table above.
