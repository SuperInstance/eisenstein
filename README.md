# eisenstein

**Exact integer arithmetic for hexagonal lattice constraints via Eisenstein integers.**

Zero unsafe code. Zero floating point. Zero dependencies. `no_std` compatible. rustc 1.75.0+.

## What This Provides

- **`E12`** — Eisenstein integer type `a + bω` where `ω = (-1 + √-3)/2`
  - Norm: `a² - ab + b²` (exact integer, no √-3 needed)
  - Hex distance, multiplication, D₆ symmetry
- **`HexDisk`** — Bounded hexagonal region of radius R
  - Vertex count: `3R² + 3R + 1`
  - 6 neighbors via Eisenstein units
- **`EisensteinTriple`** — Parametric generator `(m²-n², 2mn-n², m²-mn+n²)`
  - Multiplication closure: N(z₁z₂) = N(z₁)N(z₂)
  - D₆ Weyl orbit invariance
  - ~6.8× denser than Pythagorean triples (59,841 vs 10,428 at c < 65,536)

## Quick Start

```rust
use eisenstein::{E12, HexDisk, EisensteinTriple};

// Eisenstein integer
let z = E12::new(-5, 3);
assert_eq!(z.norm(), 49); // a²-ab+b² = 25+15+9 = 49

// Hex disk of radius 5
let disk = HexDisk::new(5);
assert_eq!(disk.vertex_count(), 91); // 3·25+3·5+1

// Parametric triple: m=7, n=4
let t = EisensteinTriple::new(7, 4);
assert_eq!(t.c(), 37); // m²-mn+n² = 49-28+16 = 37
```

## Why Eisenstein Integers?

Eisenstein integers `Z[ω]` are the natural coordinate system for hexagonal lattices:

1. **6 units** = 6 hex neighbors (no lookup table needed)
2. **Norm multiplicativity** gives exact integer constraint propagation
3. **D₆ symmetry** is built into the algebra (Weyl group of A₂)
4. **~6.8× triple density** vs Pythagorean — far more solutions per search step
5. **Laman rigidity** — hex lattice has 1.5× edge redundancy (safety margin)

## Applications

- Hex grid constraint propagation (game/simulation)
- Sensor fusion on hexagonal topologies
- Safety-critical integer-only constraint checking (DO-178C compatible)
- Lattice-based cryptography (structured lattices)
- Compressed sensing on hexagonal sampling grids

## Properties Verified

| Property | Status | Method |
|----------|--------|--------|
| Norm multiplicativity | ✅ 10,000 random multiplications, zero drift | `cargo test` |
| D₆ Weyl invariance | ✅ All 6 rotations preserve norm | `cargo test` |
| Multiplication closure | ✅ 100% (210/210 Python verification) | `eisenstein_triples.py` |
| Parametric form validity | ✅ All m,n up to 9 | `eisenstein_triples.py` |
| Laman redundancy (2D) | ✅ → 1.5× as V → ∞ | `hex_zhc.py` |
| Laman redundancy (3D FCC) | ✅ → 2.0× as V → ∞ | `hex_zhc.py` |
| O(V) holonomy check | ✅ ~0.0009ms/vertex constant | `hex_zhc.py` |

## Storage

- Each coordinate: 4 bytes (i32)
- Max norm for |q|,|r| ≤ 4096: 3·4096² = 50,331,648 (fits in 26 bits, within i32 range)
- Hex disk R=36: 3,997 vertices, 11,082 edges

## License

MIT OR Apache-2.0

## Eisenstein Ecosystem

Part of the **[Eisenstein hex integer ecosystem](https://github.com/SuperInstance/eisenstein)** — exact hex arithmetic from microcontrollers to browsers to formal verification.

| Project | Description |
|---------|-------------|
| **[eisenstein](https://github.com/SuperInstance/eisenstein)** | Core Rust crate — exact hex arithmetic, zero deps |
| **[eisenstein-c](https://github.com/SuperInstance/eisenstein-c)** | Same math, for microcontrollers. 1KB `.text`. |
| **[eisenstein-wasm](https://github.com/SuperInstance/eisenstein-wasm)** | Same math, for browsers and Node.js |
| **[eisenstein-bench](https://github.com/SuperInstance/eisenstein-bench)** | Benchmark all implementations side-by-side |
| **[eisenstein-fuzz](https://github.com/SuperInstance/eisenstein-fuzz)** | Property-based fuzzing across the ecosystem |
| **[eisenstein-do178c](https://github.com/SuperInstance/eisenstein-do178c)** | DO-178C formally verified for safety-critical systems |
| **[arm-neon-eisenstein-bench](https://github.com/SuperInstance/arm-neon-eisenstein-bench)** | 4× parallel hex math on ARM NEON |
| **[hexgrid-gen](https://github.com/SuperInstance/hexgrid-gen)** | Code generation for any language in the ecosystem |
| **[constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core)** | Production constraint framework built on Eisenstein math |
| **[flux-lucid](https://github.com/SuperInstance/flux-lucid)** | Unified intent-directed ecosystem orchestrator |

**Next →** Start with the C port for embedded: **[eisenstein-c](https://github.com/SuperInstance/eisenstein-c)**
