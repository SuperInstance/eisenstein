# HexRoomMap — the MUD as an Eisenstein lattice

*2026-08-17 · the lattice cross-pollination: eisenstein × elephant.*

A MUD's world map is a hex grid. This crate now ships the map layer that
makes that literal: **`HexRoomMap`** — rooms placed on the Eisenstein
lattice, the elephant reading each room's field, and the terrain's deadband
ringing when a region of the map crosses a threshold. A war spreading
through the hexes is not a metaphor bolted on top; it is the map's own
field crossing the band, and the ring is the witness mark that rings up
the chain.

---

## The MUD is a hex lattice

Every room of the world sits at a hex center. Hex centers are exactly the
Eisenstein integers: `a + bω` where `ω = e^(2πi/3)`. The D₆ symmetry group
of `Z[ω]` — the six units `±1, ±ω, ±ω²` — *is* the six-neighbor geometry of
the grid. There is no lookup table, no trigonometry, no floating point:
the algebra does the geometry.

```
        (0,1)   (1,1)
           \   /
    (-1,0)──(0,0)──(1,0)
           /   \
       (-1,-1) (0,-1)
```

That is a radius-1 hex disk: the center plus its six D₆ neighbors. A radius-R
disk holds `3R² + 3R + 1` cells — the same count the crate's `HexDisk` has
always reported, because it is the same lattice.

## Eisenstein integers are the true hex geometry

Two quantities matter, and this crate keeps them distinct and both exact:

- **The hex distance** — the minimal number of D₆ unit steps between two
  cells. For `p = (a₁, b₁)`, `q = (a₂, b₂)`:

  ```text
  d(p, q) = max(|Δa|, |Δb|, |Δa − Δb|)
  ```

  Every unit step has max-norm 1, and any point is reachable in exactly that
  many steps (`b` steps of `(1,1)` plus `a − b` steps of `(1,0)` for `a ≥ b ≥ 0`),
  so this is the true lattice distance. Neighbors are always at distance 1;
  the radius-R disk is a genuine hexagon of `3R² + 3R + 1` cells. This is the
  distance a MUD needs: **six neighbors, not eight** — a square grid's
  Chebyshev/Euclidean distances would lie about adjacency.

- **The Eisenstein norm** — `a² − ab + b²` of the difference, exposed as
  `norm_distance`. On the standard embedding `z = a + bω` this is the
  *squared Euclidean* distance, the exact integer quantity the field math
  consumes. It is not the hex distance (the unit `(1,1)` has norm 1; the
  pair `(1,0)`, `(0,1)` has norm 1 with hex distance 2 between them) — use
  `distance` for steps, `norm_distance` for the geometry.

> **A note on the crate's `E12::hex_distance`.** It uses the axial
> `(|q|+|r|+|q+r|)/2` formula, which belongs to a different axial neighbor
> convention: under its own `E12::directions()` it reports the neighbor
> `(1,1)` at distance 2 and `(3,−2)` at 3. The map deliberately uses the
> lattice-correct metric above so that adjacency, distance, paths, and disks
> all agree. Both are exact; only one makes neighbors distance 1.

## The API

```rust
use eisenstein::{HexRoomMap, RoomField};

let mut map = HexRoomMap::new();
map.add_room((0, 0), "The Tap")?;          // every room is a hex
map.add_room((1, 0), "the Docks")?;
map.set_field((0, 0), RoomField { mood: 0.6, ..Default::default() })?;

map.neighbors((0, 0));      // [(1,0),(0,1),(1,1),(-1,0),(0,-1),(-1,-1)] — the D₆ units
map.distance((0, 0), (5, 0));   // Some(5) — true hex distance
map.path((0, 0), (5, 0));       // Some([...]) — hex BFS over occupied rooms
map.region((0, 0), 2);          // 19 cells — the hex disk
map.fields();                   // (coord, &RoomField) per read room — set_field's read-back
map.map_temperature();          // Option<f64> — the grid's aggregate field
map.deadband_ring(0.8, 0.5);    // Option<Ring> — the terrain's deadband, with the front
```

- `add_room(coord, name)` guards impossible coordinates: the map lives on the
  crate's i32 `E12` lattice, so coordinates beyond it are rejected
  (`OutOfLatticeRange`), and nameless rooms are rejected (`UnnamedRoom`).
  Every lattice operation uses checked arithmetic — nothing panics, nothing
  silently truncates.
- `path(a, b)` walks **occupied rooms only** (empty hexes are not rooms), so
  a disconnected island returns `None` — the war can't reach it — even when
  the two rooms are geometrically close. `path` is a shortest path in hex
  steps; consecutive cells are always adjacent (distance 1).
- `region(center, radius)` is the hex disk, size `3R² + 3R + 1`, translation-
  invariant across the lattice.
- `fields()` is the read-back twin of `set_field(coord, field)`: every
  `(coord, &RoomField)` the elephant has written, in coordinate order. This
  is the seam the bridge and roomd drive — fields are pushed in as plain
  numbers and read back out; the lattice layer never computes with anything
  but exact integers.
- `map_temperature()` is the mean warmth over every room the elephant has
  read; `map_panic()` is the mean stampede reading. `None` when nothing has
  been read — an unread map has no temperature.

## The elephant reads the map

The map's second layer is the elephant's: `RoomField`, the minimal Rust
mirror of the elephant package's `RoomField` — the same seven dial readings
(mood, volume, earnestness, cynicism, joke_landing, panic, presence) and
byte-for-byte the same warmth formula. `map_temperature()` is the grid's
aggregate field over its rooms: the elephant standing in the town square
and feeling the whole map at once.

The **real** elephant is wired through `bridge/hex_room_map.py`: it imports
the elephant package (`ELEPHANT_ROOT`, or the `../elephant` sibling), turns
each map room with `events` into a real elephant `Room`, and runs the real
`DialBank(DEFAULT_DIALS)` over it — the actual mood/panic/presence dials,
not the mirror. Without the elephant importable, the bridge falls back to
the mirrored readings (same formulas, documented minimal fallback).

```text
cargo run --example hex_mud -- --json > map.json
python3 bridge/hex_room_map.py --map map.json
```

## The war spreads as a deadband ring — with a front

The terrain reframing (`elephant/docs/terrain-2026-08-17.md`) says it in one
line: **a deadband rings up the chain of command.** Small moves are not
moves; when the terrain crosses the band, the witness mark that crossed
rings — to the room's host, to the foreman, to the captain.

`deadband_ring(map_field, threshold)` is that discipline on the map:

- `|map_field| < threshold` → `None`. The room breathes, the shadows
  flicker, no one is disturbed. **A stable map stays quiet.**
- `|map_field| ≥ threshold` → `Some(Ring)` — the terrain moved. The ring
  names the **largest connected region** of rooms (hex adjacency, over read
  rooms) whose own panic crossed the same threshold. That region is the war:
  the fire in the Alley, the stampede in the Tap, the panic that has reached
  the Docks and the Smithy and keeps spreading across the lattice. Isolated
  noise blips (a lone panicking hermit) lose to the burning district — the
  ring names the war, not the hermitage.
- If the aggregate crossed but no single room did (the whole map warming at
  once), the ring names every read room.
- Equal-size regions tie-break deterministically: the one with the smallest
  coordinate wins (BTreeMap iteration order).

The ring's `coords` are the region's hexes — the same coordinates a path or
a disk would use — so the chain of command knows exactly where to send help,
and the elephant knows exactly which rooms to re-read.

### The front — the ring is propagation-aware

A fight migrating hex-by-hex is a **montage sequence**, not a set of isolated
rooms: each ring is a frame, and the ring names the direction the fight is
moving. The map remembers the region the ring last named (the montage
memory, which is why `deadband_ring` takes `&mut self`), and each ring
carries a `front`: the D₆ unit the region's centroid moved along since the
previous frame — `front_direction(previous, current)`, exact integer
arithmetic. The displacement between frames is `D = S_curr·n_prev −
S_prev·n_curr` (integer, parallel to the centroid difference), and the front
is the unit `u ∈ {±1, ±ω, ±ω²}` maximizing `2·Re(D·conj(u))` — the nearest
of the six neighbors, no trig, no floats.

- The first frame of a montage has no front (`None`): a fresh blaze has no
  history to move against.
- A re-ring over an unchanged region has no front: a standing fire is not
  moving.
- When the band goes quiet the montage ends and the memory resets, so the
  next blaze starts its own sequence.
- The bridge (`bridge/hex_room_map.py`) mirrors `front_direction` exactly;
  its `deadband_ring` takes the previous frame's coords as `previous`
  (stateless function, same quantities — both sides compute the same
  numbers by design).

## The full circle

```text
hex coords ──► HexRoomMap ──► neighbors / distance / path ──► the geometry
     │
     ├──► RoomField (mirror)  ──► map_temperature()  ──► the grid's field
     │
     └──► bridge/hex_room_map.py (the real elephant) ──► real dials per room
                               │
                               ▼
                  deadband_ring(map_field, threshold)
                               │
                     quiet ────┴──── ⚡ Ring naming the region + its D₆ front
```

Eisenstein integers are the algebra of the hex grid. The elephant is the
room's temperature. The terrain is the room's truth. The deadband is the
discipline that decides when the truth must ring. Now the truth has a map,
the map has a war, and the war has a front.
