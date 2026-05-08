# eisenstein

**The keel. Everything else in the ecosystem is built on this.**

`#![no_std]`. Zero dependencies. Zero unsafe code. Exact integer arithmetic for hexagonal lattices through Eisenstein integers — `a + bω` where `ω = (-1 + √-3)/2`. The norm `a² - ab + b²` is always an integer. No floating point, no rounding, no drift. This is the algebra of the hexagonal lattice, and it's exact all the way down.

If you're building anything on a hex grid — games, simulations, sensor networks, safety-critical systems — this is where you start. The D₆ symmetry group is baked into the type system. Six Eisenstein units map to six hex neighbors. No lookup tables. No trigonometry. The math does the work.

## Why this exists

Floating-point hex arithmetic accumulates drift. Rotate a hex coordinate ten thousand times and your position is wrong. Not "close enough" — wrong. In a constraint system, that kills you. In a lockstep multiplayer game, it desyncs you. In a DO-178C safety-critical system, it grounds the aircraft.

Eisenstein integers solve this completely. The ring `Z[ω]` is the natural coordinate system for hexagonal lattices — the same way Gaussian integers are natural for square grids. Norm multiplicativity (`‖z₁·z₂‖ = ‖z₁‖·‖z₂‖`) gives you exact integer constraint propagation. The D₆ Weyl group of A₂ gives you sixfold rotational symmetry for free. And Eisenstein triples are ~6.8× denser than Pythagorean triples — 59,841 versus 10,428 at the same bound — so you find more solutions with less searching.

This crate compiles on rustc 1.75.0+, runs on bare metal, and doesn't pull in a single dependency unless you enable the optional angle-snapping feature.

## What's inside

**`E12`** — the core Eisenstein integer type. Construct from `(a, b)`, get norm, multiplication, hex distance, and all six D₆ rotations. Each coordinate is an `i32` — 4 bytes, 26 bits of headroom at radius 4096.

**`HexDisk`** — bounded hexagonal region of radius R. Contains `3R² + 3R + 1` vertices, accessible through iteration. A radius-36 disk gives you 3,997 vertices and 11,082 edges.

**`EisensteinTriple`** — parametric generator `(m²-n², 2mn-n², m²-mn+n²)`. Produces Eisenstein integer triples with guaranteed norm multiplicativity. D₆ Weyl orbit invariance holds for all parameters.

**Angle snapping** — optional feature (`snap`). Snap floating-point angles to exact Eisenstein directions. Requires `libm` (still `no_std` compatible).

## Quick start

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

## Verified properties

Every property listed here has been verified through multiple methods — unit tests, property-based fuzzing with millions of random inputs, and independent Python cross-checks.

| Property | Method | Result |
|----------|--------|--------|
| Norm multiplicativity | 10,000 random multiplications | Zero drift |
| D₆ Weyl invariance | All 6 rotations preserve norm | Verified |
| Multiplication closure | Independent Python verification (210/210) | 100% |
| Parametric form validity | All m,n up to 9 | Verified |
| Laman redundancy (2D) | Asymptotic analysis | → 1.5× as V → ∞ |
| Laman redundancy (3D FCC) | Asymptotic analysis | → 2.0× as V → ∞ |
| O(V) holonomy check | Benchmarked | ~0.0009ms/vertex constant |

For the exhaustive fuzzing results, see [eisenstein-fuzz](https://github.com/SuperInstance/eisenstein-fuzz). For benchmarks on your own hardware, see [eisenstein-bench](https://github.com/SuperInstance/eisenstein-bench).

## Applications

- Hex grid constraint propagation for games and simulations
- Sensor fusion on hexagonal topologies
- Safety-critical integer-only constraint checking (DO-178C compatible)
- Lattice-based cryptography with structured lattices
- Compressed sensing on hexagonal sampling grids

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

**Next →** Run the numbers yourself: **[eisenstein-bench](https://github.com/SuperInstance/eisenstein-bench)**
