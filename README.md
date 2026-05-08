# eisenstein

**Exact arithmetic for hexagonal coordinates. Zero drift. Zero floats. Zero dependencies.**

Eisenstein integers are the number system that describes hexagonal lattices — same one crystallographers use, same one that tiles 2D space with exact 60° rotational symmetry. This crate makes them available as a Rust type with the full algebra: norm, multiplication, rotation, hex distance, and disk iteration. All integer arithmetic. No approximations anywhere.

## Why This Exists

Floating-point hex coordinates drift after repeated rotations. The error is small per operation, but it compounds — add a thousand corrections and your heading is off by enough to matter. Eisenstein integers don't have that problem because the norm `a² − ab + b²` is always an exact integer, every rotation stays on the lattice, and the arithmetic never leaves the ring **Z[ω]**. What you compute is what you get, every time, on every device.

## Quick Start

```rust
use eisenstein::{E12, HexDisk};

// An Eisenstein integer: point (a, b) on the hexagonal lattice
let z = E12::new(-5, 3);
assert_eq!(z.norm(), 49);         // a² − ab + b² = 49, exact integer

// 60° rotation — stays on the lattice, always
let rotated = z.rotate_60();
assert_eq!(z.rotate_60().rotate_60().rotate_60(), -z);

// All points within hex radius 5
let disk = HexDisk::new(5);
assert_eq!(disk.vertex_count(), 91); // formula: 3R² + 3R + 1

// Angle snapping (optional, needs default features)
let dir = E12::snap_from_angle(17.0); // snap any angle to nearest lattice point
```

## What You Get

**E12** — the core type. Addition, multiplication, negation, conjugate, norm, hex distance, 60° rotation, the full D₆ symmetry group (6 rotations × 2 reflections). The norm is computed with integer multiplication and subtraction — no sqrt, no float, no rounding.

**HexDisk** — bounded hexagonal region of radius R. Iterates `3R² + 3R + 1` vertices in cache-friendly order. No allocation. One pass.

**EisensteinTriple** — parametric generator `(m²−n², 2mn−n², m²−mn+n²)` producing ~6.8× denser cover of 2D space than Pythagorean triples. 59,841 triples vs 10,428 at c < 65,536.

## Where It Fits

- **Hex grid games** — Civ, Factorio, wargames. Coordinates that don't drift after 10,000 rotations.
- **Deterministic lockstep multiplayer** — Same integers in, same integers out. No FPU rounding differences. No desync.
- **Sensor fusion** — Gyroscope + compass readings combine without error accumulation. Rotations that should cancel, cancel exactly.
- **Crystallography** — Eisenstein integers are the natural coordinate system for hexagonal lattices.
- **Safety-critical** — `#![no_std]`, zero `unsafe`, zero deps, zero floats. The full matrix is 600 lines of integer arithmetic.

## The Numbers

- **4 bytes per coordinate** — two i32s. F64 needs 16 bytes and still drifts.
- **Zero unsafe** — no `unsafe` in the core type. No `unsafe` in the disk iterator. No `unsafe` anywhere.
- **Zero dependencies** — `default-features = false` gives you pure integer arithmetic with no libm. The `snap` feature adds `libm` for angle snapping.
- **Zero drift** — 10,000 rotations return exactly the starting coordinate. Tested in CI on every commit.

## License

MIT OR Apache-2.0

## Eisenstein Ecosystem

This is the core crate. The same arithmetic is available across the stack:

| Project | What It Does |
|---------|-------------|
| **[eisenstein-c](https://github.com/SuperInstance/eisenstein-c)** | Same math, 1KB .text, for microcontrollers |
| **[eisenstein-wasm](https://github.com/SuperInstance/eisenstein-wasm)** | Same math, for browsers and Node.js |
| **[eisenstein-bench](https://github.com/SuperInstance/eisenstein-bench)** | Run benchmarks on your own hardware |
| **[eisenstein-fuzz](https://github.com/SuperInstance/eisenstein-fuzz)** | 13 property tests proving the math |
| **[eisenstein-do178c](https://github.com/SuperInstance/eisenstein-do178c)** | Formal verification for safety-critical use |
| **[hexgrid-gen](https://github.com/SuperInstance/hexgrid-gen)** | Code generator for hex lookup tables in any language |
| **[constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core)** | Production constraint framework on Eisenstein math |
| **[flux-lucid](https://github.com/SuperInstance/flux-lucid)** | Intent vectors, alignment, and tolerance navigation |
