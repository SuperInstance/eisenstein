# eisenstein


## Meta

**Domain:** constraint-theory
**Depends on:** —
**Depended by:** flux-lucid, constraint-theory-ecosystem
**Implements:** zero-drift-arithmetic, hexagonal-lattice, hex-room-maps
**Related:** eisenstein-c, eisenstein-wasm, eisenstein-bench


**Exact hexagonal coordinates. No floating point. No drift. No dependencies.**

![The lattice room at night — brass rings on navy, each cell an exact integer](assets/images/gallery-eisenstein.jpg)

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

**`HexRoomMap`** — the MUD as an Eisenstein lattice. Rooms placed at hex
coordinates, six D₆ neighbors per hex, true hex distance, hex-BFS paths,
hex disks, and the elephant seam: the map's aggregate field
(`map_temperature`) and the terrain's deadband (`deadband_ring`) that rings
when a region of the map crosses a threshold — a war spreading through the
hexes. See [docs/hex-room-map.md](docs/hex-room-map.md).

## Install

Add to `Cargo.toml` (the crate is `eisenstein`, version `0.3.1`, rustc 1.75+):

```toml
[dependencies]
eisenstein = "0.3"
```

Default features include `snap` (angle snapping, pulls in `libm`). For the
leanest build on bare metal, disable it:

```toml
[dependencies]
eisenstein = { version = "0.3", default-features = false }
```

Features:

| Feature | Default | What it does |
|---------|---------|--------------|
| `snap` | ✅ | `E12::snap_from_angle` — snap float angles to exact Eisenstein directions (needs `libm`, still `no_std`) |
| `std` | ❌ | enable the `std` feature for `std::error::Error` on error types |

Build, test, and verify the no-dependency guarantee:

```sh
cargo build                               # default features
cargo build --no-default-features         # truly zero deps (no libm)
cargo test                                # unit + integration tests
cargo test --no-default-features          # and without snap
cargo build --target thumbv7em-none-eabi  # cross-compile for bare metal
```

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

// The D₆ units are the six hex neighbors
for n in E12::new(1, 0).neighbors() {
    assert_eq!(n.norm(), 1);
}
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

## The MUD as a lattice

A MUD world map is a hex grid, and Eisenstein integers *are* the hex grid:
 every room is a hex center `a + bω`, and the D₆ units `±1, ±ω, ±ω²` are its
 six neighbors — six neighbors, not eight. Distance on the map is the true
 hex distance (an exact integer lattice metric, never squared-Euclidean,
 never a float). The elephant reads each room's field; when a region of the
 map crosses the deadband, the terrain rings the war's region up the chain.

```mermaid
graph LR
    A[hex coordinates<br/>(a, b) = a + bω] --> B[HexRoomMap]
    B --> C[neighbors<br/>6 D₆ units]
    B --> D[distance<br/>true hex distance]
    B --> E[path<br/>hex BFS over rooms]
    B --> F[region<br/>hex disk 3R²+3R+1]
    B --> G[RoomField per room<br/>the elephant's reading]
    G --> H[map field<br/>map_temperature / map_panic]
    H --> I{deadband_ring<br/>map_field crosses threshold?}
    I -->|no| J[quiet — stable map]
    I -->|yes| K[⚡ Ring naming<br/>the region + its D₆ front]
```

![Hex room map — the honeycomb city](assets/images/hex-room-map.png)

A room catches fire in the Alley; its field crosses the band; the ring names
the connected region the panic has reached, and the elephant's real dials
read the rooms through `bridge/hex_room_map.py`. Run it yourself:

```sh
cargo run --example hex_mud                 # the story, in the terminal
cargo run --example hex_mud -- --json | python3 bridge/hex_room_map.py --map /dev/stdin
```

The bridge needs only Python 3 (stdlib). If the elephant package is
importable — via the `ELEPHANT_ROOT` env var or a `../elephant` sibling
checkout — every room with `events` is read by the **real** `DialBank(DEFAULT_DIALS)`;
otherwise the bridge falls back to the mirrored readings the Rust map
exported (same warmth formula, documented in
[docs/hex-room-map.md](docs/hex-room-map.md)). Bridge tests run under pytest:

```sh
python3 -m pytest bridge/ -v
```

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

## The montage name

Sergei Eisenstein cut film so each shot *collides* with the next and the
collision produces the meaning neither shot holds alone. The hex room map
takes the same idea seriously as geometry: adjacency on the lattice is a
cut. A warm room next to a cold room is a montage cut — the sauna/plunge
gap the elephant measures — and the deadband rings when the contrast
spreads, because a fight migrating through the hexes is a montage
sequence, not a set of isolated rooms. The ring is propagation-aware: it
names the connected region the fight has reached *and* the front — the D₆
unit the region moved along since the last ring, exact integer arithmetic
from the centroid displacement between two frames of the montage.

## The elephant seam, concretely

```bash
# 1. Export the map as JSON (true state, exact arithmetic, no_std core)
cargo run --example hex_mud -- --json > map.json

# 2. The bridge reads it back, runs the REAL dial bank per room,
#    and computes the same two quantities the Rust mirror computes:
python3 bridge/hex_room_map.py map.json
#    map_temperature  — the grid's aggregate field (mean warmth)
#    deadband_ring    — when a region of the map crosses its threshold
```

Both sides compute the same numbers by design — Rust for the fleet's
hard layer, Python for the elephant's soft layer — so the elephant's
reading and the map's own arithmetic never disagree. JEPA correlates;
it never replaces. The lattice is the truth; the field is its temperature.

## Fleet context

Depended on by `flux-lucid` and `constraint-theory-ecosystem`; related
ports: `eisenstein-c`, `eisenstein-wasm`, `eisenstein-bench`. In the
fleet picture this is the *chart* — terrain renders the rooms, the
elephant reads their warmth, and eisenstein decides which rooms are
neighbors, so it decides which collisions exist to be read.
