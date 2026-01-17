# Vendor Patches

This project vendors and patches `curve25519-dalek` to keep SBF stack usage
within Solana limits. These changes are **intentional** and must be reviewed
in audits.

## Rationale
SBF enforces a ~4KB stack frame limit. The upstream lookup table construction
uses fixed-size arrays on the stack, which can overflow in SBF builds. We
move large tables to the heap when `alloc` is available and
`precomputed-tables` is disabled (SBF configuration), while preserving
array-backed tables for precomputed builds.

## Patch Summary
### 1) Heap-backed lookup tables (window tables)
- **File:** `vendor/curve25519-dalek/src/window.rs`
- **Change:** `LookupTable` and `NafLookupTable8` use `Box<[T]>` under
  `cfg(all(feature = "alloc", not(feature = "precomputed-tables")))`,
  otherwise remain `[T; N]`.
- **Impact:** avoids large stack frames during table construction.

### 2) AVX2 backend table construction
- **File:** `vendor/curve25519-dalek/src/backend/vector/avx2/edwards.rs`
- **Change:** `LookupTable<CachedPoint>` and `NafLookupTable8<CachedPoint>`
  build with `Vec` + `into_boxed_slice()` when `alloc` is enabled and
  `precomputed-tables` is disabled; otherwise use stack arrays.
- **Impact:** consistent heap allocation across vector backends.

### 3) IFMA backend table construction
- **File:** `vendor/curve25519-dalek/src/backend/vector/ifma/edwards.rs`
- **Change:** same as AVX2: heap-backed tables under `alloc`, arrays otherwise.

### 4) Optional variable-time double scalar multiply
- **File:** `vendor/curve25519-dalek/src/edwards.rs`
- **Change:** added `EdwardsPoint::vartime_double_scalar_mul` implemented with
  a NAF(5) window and lookup tables.
- **Impact:** supports off-chain optimizations; not required on-chain.

## Audit Notes
- These patches are limited to allocation strategy and one optional helper.
- All crypto logic remains in upstream `curve25519-dalek`.
- Any upstream updates must re-validate these changes.

## File Hashes
SHA256 hashes for the patched files are recorded in:
`docs/VENDOR_PATCHES_SHA256.txt`
