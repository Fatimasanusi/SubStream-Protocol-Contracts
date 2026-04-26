# WASM Compression for SubStream Protocol Contracts

This document describes the WASM compression implementation for SubStream Protocol Contracts, designed to reduce deployment fees on the Stellar network by optimizing contract binary sizes.

---

## Cargo Release Profile (opt-level = "z")

All size optimizations begin at compile time. The workspace `Cargo.toml` configures the `[profile.release]` section as follows:

```toml
[profile.release]
opt-level        = "z"       # Optimize for binary size (smallest output)
lto              = true      # Link-Time Optimization: cross-crate dead-code removal
codegen-units    = 1         # Single codegen unit enables maximum inlining/DCE
strip            = "symbols" # Remove debug symbols from the final binary
debug            = 0         # No debug info
debug-assertions = false     # Disable debug_assert! macros
overflow-checks  = true      # Keep arithmetic overflow checks (security requirement)
panic            = "abort"   # No unwinding machinery; saves ~10 KB
```

### Why each setting matters

| Setting | Effect on binary size |
|---|---|
| `opt-level = "z"` | Instructs LLVM to prefer size over speed at every trade-off point |
| `lto = true` | Eliminates dead code across crate boundaries; most impactful single flag |
| `codegen-units = 1` | Allows LLVM to see the whole program and inline/remove more aggressively |
| `strip = "symbols"` | Removes the symbol table; saves several KB with no runtime cost |
| `panic = "abort"` | Removes the stack-unwinding runtime (~10 KB) |
| `debug = 0` | No DWARF sections emitted |

### Security note on LTO

LTO operates purely at the IR (LLVM bitcode) level before machine-code generation. It does not reorder or merge semantically distinct functions; it only removes provably unreachable code and inlines small callees. The Soroban host interface functions (`__check_auth`, `__get_footprint`, etc.) are marked `#[no_mangle]` and are therefore never stripped. All arithmetic overflow checks are preserved because `overflow-checks = true` is set explicitly.

### Dependency audit

The contract's `Cargo.toml` has a single runtime dependency:

```toml
[dependencies]
soroban-sdk = { workspace = true }
```

`soroban-sdk` is `#![no_std]`-compatible and does not pull in `std`, `tokio`, or any I/O runtime. There are no heavy transitive dependencies to remove.

---

## Post-Build Size Assertion

`scripts/check_wasm_size.sh` runs automatically after every `make build` and fails the build if any `.wasm` file exceeds **100 KiB (102 400 bytes)**.

```
WASM size check (limit: 102400 bytes / 100 KiB)
---
OK    substream_contracts.wasm — 42316 bytes (60084 bytes under limit)
---
Size assertion PASSED.
```

The limit can be overridden without editing the script:

```bash
MAX_WASM_SIZE=51200 make build   # enforce 50 KiB limit
```

---

## Byte-Size Comparison

The table below documents the measured binary size at each optimization stage.

| Stage | Settings | Size (bytes) | Notes |
|---|---|---|---|
| Unoptimized (`dev`) | `opt-level = 0`, no strip | ~350 000 | Default `cargo build` |
| Release baseline | `opt-level = 3`, no LTO | ~85 000 | Standard release |
| **opt-level = "z" + LTO** | Full profile above | **~42 000** | Committed configuration |
| + `wasm-opt -Oz` | Post-build Binaryen pass | ~32 000 | `make build-compressed` |

> Sizes are representative estimates based on the current contract source. Run `make build` and inspect the output to get the exact figure for your build.

The 100 KiB assertion provides a ~2.4× safety margin over the current optimized size, giving room for future feature additions while still catching accidental regressions (e.g., accidentally enabling `std` or adding a heavy dependency).

---

## Overview

The WASM compression system uses `wasm-opt` from the Binaryen toolkit to apply aggressive optimizations to contract binaries, resulting in smaller WASM files that cost less to deploy on Stellar.

## Features

- **Aggressive Optimization**: Uses `wasm-opt -Oz` with additional optimization flags
- **Size Reporting**: Shows before/after file sizes and compression percentages
- **CI/CD Integration**: Automated compression in GitHub Actions
- **Flexible Configuration**: Customizable optimization levels and output directories
- **Multiple Contracts**: Handles all WASM files in the target directory

## Tools Used

- **Binaryen**: Provides `wasm-opt` for WASM optimization
- **Stellar CLI**: Builds the contracts to WASM
- **Make**: Automation of build and compression steps

## Installation

### Local Development

1. Install Binaryen:
   ```bash
   # macOS
   brew install binaryen
   
   # Ubuntu/Debian
   sudo apt-get install binaryen
   
   # Other platforms
   # Download from https://github.com/WebAssembly/binaryen/releases
   ```

2. Verify installation:
   ```bash
   wasm-opt --version
   ```

### CI/CD

The GitHub Actions workflow automatically installs Binaryen for WASM optimization.

## Usage

### Command Line

#### Using the Makefile (Recommended)

```bash
# Build and compress in one step
cd contracts/substream_contracts
make build-compressed

# Or build first, then compress
make build
make build-compressed
```

#### Using the Compression Script

```bash
# Basic usage
./scripts/compress_wasm.sh

# With custom options
./scripts/compress_wasm.sh \
  --contract-dir contracts/substream_contracts \
  --output-dir target/compressed \
  --optimization-level Oz

# Show help
./scripts/compress_wasm.sh --help
```

### Optimization Levels

Available optimization levels for `wasm-opt`:

- `O0`: No optimization (fastest compilation)
- `O1`: Basic optimization
- `O2`: More optimization
- `O3`: Aggressive optimization
- `Os`: Optimize for size
- `Oz`: Optimize for size aggressively (recommended for deployment)

### Optimization Flags Used

The compression applies these optimization flags:

- `-Oz`: Aggressive size optimization
- `--vacuum`: Remove redundant items
- `--dae`: Dead code elimination
- `--remove-unused-names`: Remove unused names
- `--remove-unused-types`: Remove unused types
- `--merge-blocks`: Merge blocks
- `--simplify-locals`: Simplify local variables
- `--coalesce-locals`: Coalesce local variables

## File Structure

```
contracts/substream_contracts/
├── target/
│   ├── wasm32v1-none/release/
│   │   └── substream_contracts.wasm     # Original WASM
│   └── compressed/
│       └── substream_contracts.optimized.wasm  # Compressed WASM
├── Makefile                             # Build automation
└── src/
    └── lib.rs                           # Contract source
```

## CI/CD Integration

The GitHub Actions workflow (`.github/workflows/test.yml`) includes:

1. **Binaryen Installation**: Installs `wasm-opt` and related tools
2. **Contract Building**: Builds the contract using Stellar CLI
3. **WASM Compression**: Runs the compression process
4. **Artifact Upload**: Saves compressed WASM files as workflow artifacts

### Workflow Steps

```yaml
- name: Install Binaryen for WASM optimization
  run: |
    sudo apt-get update
    sudo apt-get install -y binaryen

- name: Build and Compress WASM
  run: |
    cd contracts/substream_contracts
    make build-compressed

- name: Upload Compressed WASM files
  uses: actions/upload-artifact@v3
  with:
    name: compressed-wasm
    path: contracts/substream_contracts/target/compressed/
```

## Performance Impact

### Typical Compression Results

Based on similar Stellar contracts, you can expect:

- **Size Reduction**: 10-30% smaller WASM files
- **Deployment Cost**: Proportional reduction in deployment fees
- **Runtime Performance**: Minimal to no impact on execution speed
- **Gas Costs**: No increase in transaction gas costs

### Example Output

```
🚀 SubStream Protocol WASM Compression Script
=============================================
🔨 Building contract...
📦 Compressing WASM files with optimization level: Oz
  Optimizing substream_contracts.wasm...
    ✅ Original: 45678 bytes, Compressed: 34234 bytes, Reduction: 25%
🎉 Compression complete!
📊 Summary:
   Total original size: 45678 bytes
   Total compressed size: 34234 bytes
   Total reduction: 25%
   Compressed files saved to: target/compressed
✨ Done! Your optimized WASM files are ready for deployment.
```

## Best Practices

### Development Workflow

1. **Regular Builds**: Use `make build` during development
2. **Pre-deployment**: Always use `make build-compressed` before deployment
3. **Size Monitoring**: Track compression ratios over time
4. **Testing**: Deploy compressed WASM to testnet first

### Optimization Tips

1. **Code Review**: Smaller source code often results in smaller WASM
2. **Dependencies**: Minimize external dependencies
3. **Feature Flags**: Use conditional compilation for unused features
4. **Profile**: Use `cargo bloat` to identify large dependencies

## Troubleshooting

### Common Issues

#### wasm-opt not found
```bash
# Install Binaryen
brew install binaryen  # macOS
sudo apt-get install binaryen  # Ubuntu
```

#### Build fails
```bash
# Check Rust targets
rustup target add wasm32v1-none
rustup target add wasm32-unknown-unknown
```

#### Permission denied
```bash
# Make script executable
chmod +x scripts/compress_wasm.sh
```

### Debug Mode

For debugging, use lower optimization levels:

```bash
./scripts/compress_wasm.sh --optimization-level O1
```

## Integration with Deployment

### Manual Deployment

```bash
# Build compressed WASM
make build-compressed

# Deploy using compressed file
stellar contract deploy \
  --wasm-file target/compressed/substream_contracts.optimized.wasm \
  --source-account your_account \
  --network testnet
```

### Automated Deployment

The compressed WASM files can be automatically deployed using the workflow artifacts:

1. Download `compressed-wasm` artifact from GitHub Actions
2. Extract the optimized WASM files
3. Deploy using your preferred deployment tool

## Contributing

When contributing to the compression system:

1. Test compression ratios with your changes
2. Verify that compressed contracts still function correctly
3. Update documentation for any new optimization flags
4. Consider the impact on deployment costs

## License

This WASM compression implementation is part of the SubStream Protocol Contracts project and follows the same license terms.
