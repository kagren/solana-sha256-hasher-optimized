# Overview
Fork of the sha256-hasher from the solana-sdk that optimizes the default SHA-NI implementation for single hashes of 32 bytes. Since this is quite common in the agave validator, using this crate as a drop in replacement for the defualt solana-sha256-hasher package should lead to improved performande.

NOTE: Only works on CPUs with the SHA-NI instruction set, e.g. AMD Zen3 and Zen4.

# Technical Details
SHA256 operates on block sizes of 64 bytes (512 bits), so when hashing something smaller (32 bytes in our use case), the input block will be padded with 32 bytes to make a full 64 byte block. The padding applied is deterministic for the first block, and as such we can precompute and optimize the SHA-NI code slightly as long as we are hashing 32 bytes and only a single block.

If multiple blocks are hashed, or larger input sizes than 32, then the default implentation (same as in solana-sha256-hasher) is used. On-chain SHA256 calculation is unchanged.

# Performance Benchmark
Benchmarking this in a real work setting is challenging, but there is a simple test in lib.rs that will verify the correctness of the implementation as well as do a simple benchmark. To run: ``cargo test --release -- --nocapture``

The implementation will typically increase the "PoH speed check" rate that the agave validator does at startup with about 10-20%. Has been tested in AMD 9254, AMD 9275f, Threadripper 7965wx.

# Installation in Agave or Jito
Modify the main Cargo.toml file in the Agave (or Jito) codebase:

1. Locate the patch section in Cargo.toml: ``[patch.crates-io]``
2. Add ``solana-sha256-hasher = { path = "../solana-sha256-hasher-optimized" }`` in this section and modify the path to where you have placed this repo
3. Rebuild validator client

# Disclaimer

This code is released under the [Creative Commons CC0 1.0 Universal License](https://creativecommons.org/publicdomain/zero/1.0/).

THE SOFTWARE IS PROVIDED “AS IS,” WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES, OR OTHER LIABILITY ARISING FROM, OUT OF, OR IN CONNECTION WITH THE SOFTWARE OR ITS USE.
