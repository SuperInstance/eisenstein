# Contributing to eisenstein

## What This Crate Is

`eisenstein` provides exact integer arithmetic for hexagonal lattices via Eisenstein integers `Z[ω]`. It's `#![no_std]`, zero-dependency (except optional `libm` for angle snapping), and designed for safety-critical systems where floating-point drift is unacceptable.

If you're building anything on a hex grid — games, simulations, sensor networks, constraint systems — this is the foundation.

## Development Setup

```bash
git clone https://github.com/SuperInstance/eisenstein.git
cd eisenstein
cargo build
cargo test
```

### Prerequisites

- Rust 1.75.0+ (uses `#![no_std]` core features)
- No system dependencies required for core crate
- `libm` is pulled in only when the `snap` feature is enabled (default)

## Running Tests

```bash
# All tests (lib + integration + doc-tests)
cargo test

# Just the algebraic property tests
cargo test --test algebraic_properties

# Just the edge case tests
cargo test --test edge_cases

# Doc tests only
cargo test --doc

# Run a specific test
cargo test ring_axioms::addition_is_associative
```

### Test Organization

| File | What It Tests |
|------|--------------|
| `src/lib.rs` (inline `#[cfg(test)]`) | Core functionality: construction, accessors, basic ops |
| `tests/edge_cases.rs` | Boundary conditions, algebraic properties of individual operations |
| `tests/algebraic_properties.rs` | Deep structural tests: ring axioms, Euclidean domain, D₆ symmetry, norm multiplicativity (exhaustive over small domains) |

The algebraic property tests are structured as mathematical theorems — each test verifies a mathematical statement, not just a function's output.

## Code Style

- `#![no_std]` is non-negotiable for the core crate. No `std` or `alloc` in `src/lib.rs` outside of `#[cfg(test)]` and explicit `extern crate alloc` blocks.
- Every public function needs a doc comment with at least one example.
- Every new feature needs test coverage — aim for algebraic property tests, not just example tests.
- Use conventional commits: `feat:`, `fix:`, `test:`, `docs:`, `chore:`, `refactor:`.
- Zero `unsafe` code. This is a safety-critical library.
- i32 for coordinates. The norm uses i64 intermediates and returns u64. This gives 26 bits of headroom at radius 4096.

## Architecture

### Core Types

- **`E12`** — Eisenstein integer `a + bω`. Supports `+`, `-`, `*`, conjugation, D₆ rotations, Euclidean division, GCD, divisibility checking.
- **`HexDisk`** — Bounded hexagonal region. Iterable. `3R² + 3R + 1` vertices at radius R.
- **`EisensteinTriple`** — Parametric triples `(a, b, c)` with `a² - ab + b² = c²`. ~6.8× denser than Pythagorean triples.

### Key Design Decisions

1. **Integer-only arithmetic.** No floats in the core. The norm `a² - ab + b²` is always exact. This is the whole point.
2. **`i32` coordinates.** Large enough for practical use (±2 billion), small enough that `i64` intermediates never overflow.
3. **`u64` norm.** The norm is always non-negative. `u64` makes this a type-level guarantee.
4. **Optional `snap` feature.** Angle snapping requires trig, which needs `libm`. It's behind a feature flag so the core crate stays dependency-free.

### The Math

Eisenstein integers form the ring `Z[ω]` where `ω = e^{2πi/3}` is a primitive cube root of unity. Key identities:

- `1 + ω + ω² = 0`
- `ω² = -1 - ω = conj(ω)`
- Norm: `N(a + bω) = a² - ab + b²` (always non-negative, multiplicative)
- `Z[ω]` is a Euclidean domain (and therefore a UFD and PID)
- The 6 units are: `±1, ±ω, ±ω²`
- The D₆ point group acts on `Z[ω]` by multiplication by units

## Pull Request Checklist

- [ ] `cargo test` passes (all 200+ tests)
- [ ] New code has test coverage
- [ ] No `unsafe` code added
- [ ] No new dependencies added to the core crate
- [ ] `#![no_std]` compatibility maintained
- [ ] Documentation updated if behavior changed
- [ ] Commit messages follow conventional commits

## Fleet Context

This crate is part of the SuperInstance fleet. It's depended on by:
- `flux-lucid` — constraint systems
- `constraint-theory-ecosystem` — constraint theory research
- `eisenstein-c`, `eisenstein-wasm`, `eisenstein-bench` — language ports and benchmarks

Related crates:
- `eisenstein-c` — C bindings
- `eisenstein-wasm` — WebAssembly bindings
- `eisenstein-bench` — benchmarks
