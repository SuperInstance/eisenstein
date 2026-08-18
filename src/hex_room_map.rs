//! HexRoomMap — the MUD as an Eisenstein lattice, the elephant reading the hexes.
//!
//! A MUD world map is a hex grid. Every hex center is an Eisenstein integer
//! `a + bω`; the D₆ symmetry group of `Z[ω]` *is* the six-neighbor geometry of
//! the grid. This module turns that algebra into a room map:
//!
//! - **rooms** live at lattice coordinates `(a, b)` (i64, wide enough for any
//!   map; the crate's core `E12` type is i32-based, so coordinates outside the
//!   i32 lattice are rejected as out-of-lattice / impossible),
//! - **neighbors** are the six D₆ unit directions (reused from [`crate::E12::directions`]),
//! - **distance** is the true hex distance — an exact integer lattice metric,
//!   never squared-Euclidean, never a float,
//! - **path** is a hex BFS across occupied rooms,
//! - **region** is the hex disk (3R² + 3R + 1 cells),
//! - **map_temperature** and **deadband_ring** are the elephant seam: the
//!   aggregate field over the rooms, and the terrain's deadband ringing when a
//!   region of the map crosses a threshold (a war spreading through the hexes).
//!   The ring is **propagation-aware**: it remembers the last region it named,
//!   so a fight migrating hex-by-hex reads as a montage sequence with a
//!   **front** — the D₆ unit the region moved along ([`front_direction`]),
//!   exact integer arithmetic, not a set of isolated rooms.
//!
//! # The elephant seam
//!
//! The elephant (the `elephant` Python package, sibling repo) reads *any* room
//! through its `RoomField` abstraction: warmth, κ, and the dial readings
//! (mood, volume, earnestness, cynicism, joke_landing, panic, presence). This
//! module ships a minimal Rust mirror of that field ([`RoomField`], identical
//! warmth formula — the documented fallback), plus a Python bridge
//! (`bridge/hex_room_map.py`) that runs the **real** elephant dials over the
//! map when the elephant is importable (`ELEPHANT_ROOT` or `../elephant`
//! sibling). `map_temperature()` and `deadband_ring()` are the same functions
//! on both sides: the grid's aggregate field, and the ring that rings up the
//! chain when a region crosses the deadband.
//!
//! # The distance, precisely
//!
//! For coordinates `p = (a₁, b₁)`, `q = (a₂, b₂)` the hex distance is
//!
//! ```text
//! d(p, q) = max(|Δa|, |Δb|, |Δa − Δb|),   Δa = a₁ − a₂, Δb = b₁ − b₂
//! ```
//!
//! This is the minimal number of D₆ unit steps between two hexes — provably:
//! every unit step has max-norm 1, and a point with `a ≥ b ≥ 0` is reached in
//! exactly `a` steps (`b` steps of `(1,1)` plus `a − b` steps of `(1,0)`). The
//! disk of radius R under this metric contains `3R² + 3R + 1` cells, matching
//! the crate's [`crate::HexDisk`]. (The crate's `E12::hex_distance` uses the
//! axial `(|q|+|r|+|q+r|)/2` formula, which belongs to a different axial
//! neighbor convention and reports its own neighbor `(1,1)` at distance 2 —
//! inconsistent with `E12::directions()`. This map uses the lattice-correct
//! metric above so that neighbors are always at distance 1.)
//!
//! [`RoomField::norm_distance`] additionally exposes the Eisenstein norm
//! `a² − ab + b²` of the difference — the *squared* Euclidean distance on the
//! standard embedding — which is the quantity the elephant's field math
//! actually consumes.

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::E12;

/// The six D₆ unit directions of the Eisenstein lattice, as i64 coordinates.
///
/// These are `±1, ±ω, ±ω²` — the units of `Z[ω]`, i.e. the six neighbors of
/// every hex. Reused from the crate's own D₆ symmetry code
/// ([`E12::directions`]) so the map and the ring stay in lockstep.
pub fn hex_directions() -> [(i64, i64); 6] {
    let dirs = E12::directions();
    [
        (dirs[0].a() as i64, dirs[0].b() as i64),
        (dirs[1].a() as i64, dirs[1].b() as i64),
        (dirs[2].a() as i64, dirs[2].b() as i64),
        (dirs[3].a() as i64, dirs[3].b() as i64),
        (dirs[4].a() as i64, dirs[4].b() as i64),
        (dirs[5].a() as i64, dirs[5].b() as i64),
    ]
}

/// True hex distance on the Eisenstein lattice: the minimal number of D₆ unit
/// steps between two cells.
///
/// `d = max(|Δa|, |Δb|, |Δa − Δb|)` — an exact integer, never squared
/// Euclidean, never a float. Returns `None` only if the difference overflows
/// i64 (impossible for any real map).
#[inline]
pub fn hex_distance(a: (i64, i64), b: (i64, i64)) -> Option<u64> {
    let da = a.0.checked_sub(b.0)?;
    let db = a.1.checked_sub(b.1)?;
    let ds = da.checked_sub(db)?; // Δa − Δb
    let dist = da.abs().max(db.abs()).max(ds.abs());
    Some(dist as u64)
}

/// The Eisenstein norm of the difference `a − b`: `Δa² − Δa·Δb + Δb²`.
///
/// This is the *squared* Euclidean distance on the standard embedding
/// `z = a + bω` — the exact integer quantity the elephant's field math
/// consumes. It is **not** the hex distance (e.g. the unit `(1,1)` has norm 1
/// but the lattice neighbors `(1,0)` and `(0,1)` at hex distance 2 from each
/// other are norm 1 too); use [`hex_distance`] for steps. Computed in i128,
/// so it never overflows.
#[inline]
pub fn norm_distance(a: (i64, i64), b: (i64, i64)) -> Option<u64> {
    let da = a.0.checked_sub(b.0)? as i128;
    let db = a.1.checked_sub(b.1)? as i128;
    let n = da * da - da * db + db * db;
    if n < 0 || n > u64::MAX as i128 {
        None
    } else {
        Some(n as u64)
    }
}

/// The front of a spreading fight — the D₆ direction a region moved along.
///
/// A fight migrating hex-by-hex is a montage sequence, and the front is the
/// direction the region travels between two frames of it. Each frame is just
/// the list of hex coordinates the ring named (the region now, and the region
/// the ring named last time); the front is the D₆ unit nearest the
/// displacement between the frames' centroids — exact integer arithmetic, no
/// trig, no drift.
///
/// Concretely, with sums `S_prev`, `S_curr` and counts `n_prev`, `n_curr`,
/// the displacement is `D = S_curr·n_prev − S_prev·n_curr` — an integer
/// vector parallel to `centroid(curr) − centroid(prev)` (scaled by
/// `n_prev·n_curr`, which preserves direction). The front is the unit
/// `u ∈ {±1, ±ω, ±ω²}` maximizing `2·Re(D·conj(u))`, i.e.
/// `2x·a − x·b − y·a + 2y·b` for `D = (x, y)` — the unit nearest `D` on the
/// standard embedding, computed exactly. Ties (a displacement exactly between
/// two units) break to the first unit in [`hex_directions`] order:
/// deterministic, like every tie in this crate.
///
/// `None` when either frame is empty, when the displacement is zero (a
/// settled blaze is not moving — no montage, no front), or when the sums
/// overflow (impossible for any coordinate a map can hold).
pub fn front_direction(previous: &[(i64, i64)], current: &[(i64, i64)]) -> Option<(i64, i64)> {
    if previous.is_empty() || current.is_empty() {
        return None;
    }
    let mut prev = (0i128, 0i128, 0i128); // (Σa, Σb, n)
    for &(a, b) in previous {
        prev.0 = prev.0.checked_add(a as i128)?;
        prev.1 = prev.1.checked_add(b as i128)?;
        prev.2 += 1;
    }
    let mut curr = (0i128, 0i128, 0i128);
    for &(a, b) in current {
        curr.0 = curr.0.checked_add(a as i128)?;
        curr.1 = curr.1.checked_add(b as i128)?;
        curr.2 += 1;
    }
    // D = S_curr·n_prev − S_prev·n_curr  ∥  centroid(curr) − centroid(prev)
    let dx = curr
        .0
        .checked_mul(prev.2)?
        .checked_sub(prev.0.checked_mul(curr.2)?)?;
    let dy = curr
        .1
        .checked_mul(prev.2)?
        .checked_sub(prev.1.checked_mul(curr.2)?)?;
    if dx == 0 && dy == 0 {
        return None;
    }
    let mut best = None;
    let mut best_align = i128::MIN;
    for &(a, b) in hex_directions().iter() {
        // 2·Re(D·conj(u)) = 2x·a − x·b − y·a + 2y·b, exact in integers.
        let (a, b) = (a as i128, b as i128);
        let align = dx
            .checked_mul(a.checked_mul(2)?)?
            .checked_sub(dx.checked_mul(b)?)?
            .checked_sub(dy.checked_mul(a)?)?
            .checked_add(dy.checked_mul(b.checked_mul(2)?)?)?;
        if align > best_align {
            best_align = align;
            best = Some((a, b));
        }
    }
    best.map(|(a, b)| (a as i64, b as i64))
}

/// Why a coordinate is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapError {
    /// Coordinates outside the crate's i32 E12 lattice (|a| or |b| > i32::MAX)
    /// cannot live on the map — the map bridges to `E12`, `HexDisk`, and the
    /// elephant bridge, all of which assume the i32 lattice.
    OutOfLatticeRange,
    /// A room needs a name (the map is a MUD, not a grid).
    UnnamedRoom,
    /// The coordinate is not a room on this map.
    RoomNotFound,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapError::OutOfLatticeRange => write!(
                f,
                "coordinate is outside the i32 E12 lattice (impossible/NaN-like coordinate)"
            ),
            MapError::UnnamedRoom => write!(f, "a room must have a name"),
            MapError::RoomNotFound => write!(f, "no room at that coordinate"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MapError {}

/// A room's field — the minimal Rust mirror of the elephant's `RoomField`.
///
/// Seven dial readings, one per elephant sense (mood, volume, earnestness,
/// cynicism, joke_landing, panic, presence). `warmth()` reproduces the
/// elephant's exact formula so the Rust fallback and the real elephant agree
/// to the bit: mood & joke_landing run [-1, +1]; the rest run [0, 1] and are
/// re-centered; panic and cynicism are cold; presence and earnestness are
/// warm; volume is heat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomField {
    /// Warm/cold valence of the room, [-1, +1]. Default 0.0.
    pub mood: f64,
    /// How loud the room is, [0, 1]. Default 0.5.
    pub volume: f64,
    /// How much the room means it, [0, 1]. Default 0.5.
    pub earnestness: f64,
    /// How much the room rolls its eyes, [0, 1]. Default 0.5.
    pub cynicism: f64,
    /// Did the joke land? [-1, +1]. Default 0.0.
    pub joke_landing: f64,
    /// The stampede sense — fire in the room, [0, 1]. Default 0.0.
    pub panic: f64,
    /// Pheromone trace — who's been here, [0, 1]. Default 0.5.
    pub presence: f64,
}

impl Default for RoomField {
    fn default() -> Self {
        Self {
            mood: 0.0,
            volume: 0.5,
            earnestness: 0.5,
            cynicism: 0.5,
            joke_landing: 0.0,
            panic: 0.0,
            presence: 0.5,
        }
    }
}

impl RoomField {
    /// Create a field from the seven elephant dial readings, in dial order.
    pub fn new(
        mood: f64,
        volume: f64,
        earnestness: f64,
        cynicism: f64,
        joke_landing: f64,
        panic: f64,
        presence: f64,
    ) -> Self {
        Self {
            mood,
            volume,
            earnestness,
            cynicism,
            joke_landing,
            panic,
            presence,
        }
    }

    /// The felt temperature — identical formula to the elephant's
    /// `RoomField.warmth()`: ~[-1, +1].
    #[inline]
    pub fn warmth(&self) -> f64 {
        0.30 * self.mood
            + 0.15 * self.joke_landing
            + 0.10 * (self.earnestness - 0.5) * 2.0
            + 0.10 * (self.presence - 0.5) * 2.0
            + 0.10 * (self.volume - 0.5) * 2.0
            - 0.15 * self.cynicism
            - 0.10 * self.panic
    }

    /// The stampede sense, [0, 1]. Convenience accessor for the deadband.
    #[inline]
    pub fn panic(&self) -> f64 {
        self.panic
    }
}

/// A deadband ring — the terrain ringing up the chain of command.
///
/// When the map's field crosses a threshold (a panic spreading, a war moving
/// through the hexes), the elephant doesn't whisper — it rings: the ring
/// names the region that crossed, the coordinates of its hexes, the center of
/// the region, the map field that triggered it, the threshold it crossed,
/// and — because the ring is propagation-aware — the **front**: the D₆ unit
/// the region moved along since the ring last fired (see [`front_direction`]).
/// On a stable map, nothing rings.
#[derive(Debug, Clone, PartialEq)]
pub struct Ring {
    /// The room names of the region that crossed the deadband (in coordinate
    /// order).
    pub region: Vec<String>,
    /// The hex coordinates of the region's rooms.
    pub coords: Vec<(i64, i64)>,
    /// The first coordinate of the region (its anchor hex).
    pub center: (i64, i64),
    /// The D₆ front: the unit direction the region has moved along since the
    /// ring last named one — where the fight is heading. `None` on the
    /// montage's first frame (a fresh blaze has no history to move against)
    /// and when the region did not move (a settled blaze is not a montage).
    /// Always one of [`hex_directions`].
    pub front: Option<(i64, i64)>,
    /// The map field (aggregate temperature) that crossed the threshold.
    pub map_field: f64,
    /// The deadband threshold crossed.
    pub threshold: f64,
}

/// HexRoomMap — rooms placed on the Eisenstein lattice.
///
/// Every room is a hex; every hex is an Eisenstein integer. The map owns two
/// layers: the rooms (`coord -> name`, the MUD's geography) and the fields
/// (`coord -> RoomField`, the elephant's readings of each room), plus the
/// montage memory (`last_blaze`, the region the ring named last time — what
/// makes the deadband propagation-aware). Distance, adjacency, disks, paths,
/// and the ring's front are all exact integer lattice math — no drift, no
/// trigonometry.
#[derive(Debug, Clone, Default)]
pub struct HexRoomMap {
    rooms: BTreeMap<(i64, i64), String>,
    fields: BTreeMap<(i64, i64), RoomField>,
    last_blaze: Option<Vec<(i64, i64)>>,
}

impl HexRoomMap {
    /// A new, empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a room at an Eisenstein coordinate.
    ///
    /// Guards: the coordinate must lie on the crate's i32 E12 lattice
    /// (`OutOfLatticeRange` otherwise — the impossible/NaN-like case; i64 has
    /// no NaN, so the lattice range is the only impossible coordinate), and
    /// the name must be non-empty. Adding a room at an existing coordinate
    /// overwrites the name (the MUD repurposed the hex).
    pub fn add_room(&mut self, coord: (i64, i64), name: &str) -> Result<(), MapError> {
        if coord.0 > i32::MAX as i64
            || coord.0 < i32::MIN as i64
            || coord.1 > i32::MAX as i64
            || coord.1 < i32::MIN as i64
        {
            return Err(MapError::OutOfLatticeRange);
        }
        if name.is_empty() {
            return Err(MapError::UnnamedRoom);
        }
        self.rooms.insert(coord, name.into());
        Ok(())
    }

    /// Give a room its field — the elephant's reading of that hex.
    ///
    /// Only existing rooms can be read; setting a field for a hex with no room
    /// is an error (`RoomNotFound`).
    pub fn set_field(&mut self, coord: (i64, i64), field: RoomField) -> Result<(), MapError> {
        if !self.rooms.contains_key(&coord) {
            return Err(MapError::RoomNotFound);
        }
        self.fields.insert(coord, field);
        Ok(())
    }

    /// The room name at a coordinate, if any.
    pub fn get(&self, coord: (i64, i64)) -> Option<&str> {
        self.rooms.get(&coord).map(|s| s.as_str())
    }

    /// Is there a room at this hex?
    pub fn contains(&self, coord: (i64, i64)) -> bool {
        self.rooms.contains_key(&coord)
    }

    /// Number of rooms on the map.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Iterate over `(coord, name)` for every room, in coordinate order.
    pub fn iter(&self) -> impl Iterator<Item = ((i64, i64), &str)> {
        self.rooms.iter().map(|(&c, n)| (c, n.as_str()))
    }

    /// Iterate over `(coord, &RoomField)` for every room the elephant has
    /// read, in coordinate order — the read-back twin of [`set_field`].
    ///
    /// Rooms without a field are not included: an unread room has no reading.
    /// The bridge layer drives the elephant from exactly this seam (the map
    /// stores the fields; floats never leave the struct they arrived in).
    pub fn fields(&self) -> impl Iterator<Item = ((i64, i64), &RoomField)> {
        self.fields.iter().map(|(&c, f)| (c, f))
    }

    /// The six hex neighbors of a coordinate — the D₆ unit directions.
    ///
    /// Reuses the crate's `E12::directions()` (the six units of `Z[ω]`), so
    /// map adjacency and the ring's symmetry group are literally the same
    /// object. Returns `None` only if the coordinate arithmetic overflows i64
    /// (impossible for coordinates on the map).
    pub fn neighbors(&self, coord: (i64, i64)) -> Option<[(i64, i64); 6]> {
        let mut out = [(0i64, 0i64); 6];
        for (i, (da, db)) in hex_directions().iter().enumerate() {
            out[i] = (coord.0.checked_add(*da)?, coord.1.checked_add(*db)?);
        }
        Some(out)
    }

    /// True hex distance between two coordinates — the Eisenstein lattice
    /// metric, exact and integer. See [`hex_distance`].
    pub fn distance(&self, a: (i64, i64), b: (i64, i64)) -> Option<u64> {
        hex_distance(a, b)
    }

    /// The Eisenstein norm of the difference — the squared Euclidean distance
    /// on this embedding, exact. See [`norm_distance`].
    pub fn norm_distance(&self, a: (i64, i64), b: (i64, i64)) -> Option<u64> {
        norm_distance(a, b)
    }

    /// The hex disk: every lattice cell within `radius` hex-steps of `center`.
    ///
    /// Size is exactly `3R² + 3R + 1` — the same count as the crate's
    /// [`crate::HexDisk`], because the metric is the same hex lattice. Pure
    /// i64 lattice math; returns `None` if the disk is too large to size
    /// (checked arithmetic — impossible for any real map) or the coordinate
    /// arithmetic overflows. The disk is O(R²); keep radii sane for your map.
    pub fn region(&self, center: (i64, i64), radius: u32) -> Option<Vec<(i64, i64)>> {
        let r = radius as i64;
        let size = (radius as u128)
            .checked_mul(radius as u128)?
            .checked_mul(3)?
            .checked_add((3 * radius) as u128)?
            .checked_add(1)?;
        let size = usize::try_from(size).ok()?;
        let mut out = Vec::with_capacity(size);
        for da in -r..=r {
            for db in -r..=r {
                let p = (center.0.checked_add(da)?, center.1.checked_add(db)?);
                if hex_distance(center, p)? <= radius as u64 {
                    out.push(p);
                }
            }
        }
        Some(out)
    }

    /// A hex BFS path between two rooms, stepping only through occupied hexes.
    ///
    /// Both endpoints must be rooms on the map; the returned path is a
    /// shortest path in hex steps where every consecutive pair is adjacent
    /// (hex distance 1) and every cell is a room. `None` if an endpoint is
    /// missing or the rooms are disconnected (an island the war can't reach).
    pub fn path(&self, a: (i64, i64), b: (i64, i64)) -> Option<Vec<(i64, i64)>> {
        if !self.rooms.contains_key(&a) || !self.rooms.contains_key(&b) {
            return None;
        }
        if a == b {
            return Some(vec![a]);
        }
        let mut prev: BTreeMap<(i64, i64), (i64, i64)> = BTreeMap::new();
        let mut visited: BTreeSet<(i64, i64)> = BTreeSet::new();
        let mut queue: VecDeque<(i64, i64)> = VecDeque::new();
        visited.insert(a);
        queue.push_back(a);
        while let Some(c) = queue.pop_front() {
            let ns = self.neighbors(c)?;
            for n in ns {
                if !self.rooms.contains_key(&n) || visited.contains(&n) {
                    continue;
                }
                prev.insert(n, c);
                if n == b {
                    let mut path = vec![n];
                    let mut cur = n;
                    while cur != a {
                        cur = *prev.get(&cur)?;
                        path.push(cur);
                    }
                    path.reverse();
                    return Some(path);
                }
                visited.insert(n);
                queue.push_back(n);
            }
        }
        None
    }

    /// The map's temperature: the grid's aggregate field over its rooms.
    ///
    /// Mean warmth across every room that has a field (unread rooms — no
    /// field — are simply not part of the reading). `None` when no room has
    /// been read yet: the elephant has nothing to feel.
    pub fn map_temperature(&self) -> Option<f64> {
        let mut sum = 0.0f64;
        let mut n = 0u64;
        for f in self.fields.values() {
            sum += f.warmth();
            n += 1;
        }
        if n == 0 {
            None
        } else {
            Some(sum / n as f64)
        }
    }

    /// The map's aggregate panic — the stampede sense over the whole grid.
    ///
    /// Mean panic across read rooms; `None` when nothing has been read. This
    /// is the reading that makes a war visible: it climbs as the hot region
    /// spreads.
    pub fn map_panic(&self) -> Option<f64> {
        let mut sum = 0.0f64;
        let mut n = 0u64;
        for f in self.fields.values() {
            sum += f.panic;
            n += 1;
        }
        if n == 0 {
            None
        } else {
            Some(sum / n as f64)
        }
    }

    /// The terrain's deadband: does the map's field cross the threshold?
    ///
    /// When `|map_field| >= threshold` the terrain has moved past the
    /// deadband — something real is happening — and the map rings: the ring
    /// names the **largest connected region** of rooms (hex adjacency, over
    /// read rooms) whose own panic has crossed the same threshold. That
    /// region is the war spreading through the hexes. If several regions tie
    /// for largest, the first maximal region in coordinate order wins
    /// (fields are iterated as a BTreeMap — deterministic, and the smallest
    /// coordinates come first). If no individual room crosses but the
    /// aggregate does (the whole map warming at once), the ring names every
    /// read room. On a stable map (`|map_field| < threshold`, or no rooms
    /// read) nothing rings — `None`.
    ///
    /// # The ring is propagation-aware
    ///
    /// The map remembers the region the ring last named (the montage
    /// memory), so this is `&mut self`: each ring compares the region it
    /// names against the previous frame and reports the **front** — the D₆
    /// unit the region moved along ([`front_direction`]). A fight migrating
    /// hex-by-hex is a montage sequence with a front, not a set of isolated
    /// rooms: frame 1 names the seed hex (`front: None` — a fresh blaze has
    /// no history), frame 2 names the region plus the direction it spread.
    /// When the band goes quiet the montage ends and the memory resets, so
    /// the next blaze starts its own sequence. A re-run over an unchanged
    /// region reports `front: None` — a standing fire is not moving.
    pub fn deadband_ring(&mut self, map_field: f64, threshold: f64) -> Option<Ring> {
        // Quiet unless the field is a real number that crossed the band:
        // NaN readings and unread maps stay quiet (NaN compares false with
        // everything, so `<` alone would let it slip through — spelled out).
        let quiet = threshold.is_nan()
            || threshold < 0.0
            || map_field.is_nan()
            || map_field.abs() < threshold
            || self.fields.is_empty();
        if quiet {
            // The band went quiet: the montage ends here. Forget the last
            // blaze so the next one starts its own sequence.
            self.last_blaze = None;
            return None;
        }
        // Flood-fill over read rooms with panic >= threshold; keep the
        // largest connected region.
        let mut best: Vec<(i64, i64)> = Vec::new();
        let mut seen: BTreeSet<(i64, i64)> = BTreeSet::new();
        for (&coord, field) in self.fields.iter() {
            if field.panic < threshold || seen.contains(&coord) {
                continue;
            }
            let mut comp: Vec<(i64, i64)> = Vec::new();
            let mut stack: VecDeque<(i64, i64)> = VecDeque::new();
            seen.insert(coord);
            stack.push_back(coord);
            while let Some(c) = stack.pop_front() {
                comp.push(c);
                if let Some(ns) = self.neighbors(c) {
                    for n in ns {
                        if seen.contains(&n) {
                            continue;
                        }
                        match self.fields.get(&n) {
                            Some(f) if f.panic >= threshold => {
                                seen.insert(n);
                                stack.push_back(n);
                            }
                            _ => {}
                        }
                    }
                }
            }
            if comp.len() > best.len() {
                best = comp;
            }
        }
        let coords = if best.is_empty() {
            self.fields.keys().copied().collect::<Vec<_>>()
        } else {
            best
        };
        let front = front_direction(self.last_blaze.as_deref().unwrap_or(&[]), &coords);
        self.last_blaze = Some(coords.clone());
        let region = coords
            .iter()
            .filter_map(|c| self.rooms.get(c).cloned())
            .collect::<Vec<_>>();
        let center = coords[0];
        Some(Ring {
            region,
            coords,
            center,
            front,
            map_field,
            threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::vec;

    #[test]
    fn hex_directions_are_the_six_units() {
        let dirs = hex_directions();
        assert_eq!(dirs.len(), 6);
        // All distinct, all at hex distance 1 from origin, all Eisenstein units.
        for (i, d) in dirs.iter().enumerate() {
            assert_eq!(hex_distance((0, 0), *d), Some(1));
            assert_eq!(norm_distance((0, 0), *d), Some(1), "unit {:?}", d);
            for (j, e) in dirs.iter().enumerate() {
                if i != j {
                    assert_ne!(d, e);
                }
            }
        }
    }

    #[test]
    fn distance_matches_known_layouts() {
        let m = HexRoomMap::new();
        assert_eq!(m.distance((0, 0), (0, 0)), Some(0));
        assert_eq!(m.distance((0, 0), (1, 0)), Some(1));
        assert_eq!(m.distance((0, 0), (0, 1)), Some(1));
        assert_eq!(m.distance((0, 0), (1, 1)), Some(1)); // a D₆ neighbor
        assert_eq!(m.distance((0, 0), (-1, -1)), Some(1));
        assert_eq!(m.distance((0, 0), (1, -1)), Some(2));
        assert_eq!(m.distance((0, 0), (2, -1)), Some(3));
        assert_eq!(m.distance((0, 0), (3, -2)), Some(5));
        // Symmetric.
        assert_eq!(m.distance((3, -2), (0, 0)), m.distance((0, 0), (3, -2)));
        // Triangle inequality on a known triangle.
        let (p, q, r) = ((0, 0), (2, 0), (3, 1));
        assert!(m.distance(p, r).unwrap() <= m.distance(p, q).unwrap() + m.distance(q, r).unwrap());
    }

    #[test]
    fn norm_distance_is_the_squared_euclidean() {
        let m = HexRoomMap::new();
        // Norm of (1,0): 1. Norm of (1,1): 1² - 1 + 1 = 1 — a unit, even though
        // hex distance to (0,0) via the lattice is 1 for both. The two metrics
        // are different animals; both exact.
        assert_eq!(m.norm_distance((0, 0), (1, 0)), Some(1));
        assert_eq!(m.norm_distance((0, 0), (1, 1)), Some(1));
        assert_eq!(m.norm_distance((0, 0), (3, -2)), Some(19)); // 9 + 6 + 4
        assert_eq!(m.norm_distance((0, 0), (1, -1)), Some(3)); // not a square — never a lattice distance
                                                               // Multiplicativity of the norm: norm(a-b) with a=2·(1,0)... check
                                                               // norm of (2,0) = 4 = 2².
        assert_eq!(m.norm_distance((0, 0), (2, 0)), Some(4));
    }

    #[test]
    fn neighbors_are_the_six_hex_neighbors() {
        let m = HexRoomMap::new();
        let ns = m.neighbors((0, 0)).unwrap();
        assert_eq!(ns.len(), 6);
        // Every neighbor is at hex distance 1 (the whole point of the D₆ set).
        for n in ns {
            assert_eq!(m.distance((0, 0), n), Some(1));
        }
        // And a room's neighbors are its coordinate + each unit direction.
        let dirs = hex_directions();
        for (i, d) in dirs.iter().enumerate() {
            assert_eq!(ns[i], *d);
        }
        // Non-origin hex has the same six-neighbor shape.
        let ns2 = m.neighbors((5, -3)).unwrap();
        for n in ns2 {
            assert_eq!(m.distance((5, -3), n), Some(1));
        }
    }

    #[test]
    fn region_is_a_hex_disk_of_the_right_size() {
        let m = HexRoomMap::new();
        for r in 0..=5u32 {
            let disk = m.region((0, 0), r).unwrap();
            assert_eq!(disk.len(), (3 * r * r + 3 * r + 1) as usize, "radius {}", r);
            for p in &disk {
                assert!(m.distance((0, 0), *p).unwrap() <= r as u64);
            }
        }
        // Radius 1 = center + its six neighbors, exactly.
        let disk1 = m.region((0, 0), 1).unwrap();
        assert_eq!(disk1.len(), 7);
        assert!(disk1.contains(&(0, 0)));
        for n in m.neighbors((0, 0)).unwrap() {
            assert!(disk1.contains(&n));
        }
        // Disks are translation-invariant in size.
        assert_eq!(
            m.region((4, 4), 3).unwrap().len(),
            m.region((0, 0), 3).unwrap().len()
        );
    }

    #[test]
    fn path_is_valid_and_adjacent() {
        let mut m = HexRoomMap::new();
        // A 7-room disk around the origin, plus a far room.
        for c in m.region((0, 0), 1).unwrap() {
            m.add_room(c, &format!("r{}{}", c.0, c.1)).unwrap();
        }
        m.add_room((5, 0), "far").unwrap();
        // Path across the disk.
        let p = m.path((0, 0), (1, 1)).unwrap();
        assert_eq!(p.first(), Some(&(0, 0)));
        assert_eq!(p.last(), Some(&(1, 1)));
        for w in p.windows(2) {
            assert_eq!(m.distance(w[0], w[1]), Some(1), "adjacent step {:?}", w);
            assert!(m.contains(w[0]) && m.contains(w[1]));
        }
        // Path from a neighbor to the far room steps through rooms; the far
        // room is 5 steps away at the lattice level and unreachable here
        // (empty hexes are not rooms).
        assert_eq!(m.path((0, 0), (5, 0)), None);
        // A line of rooms bridges it.
        for k in 1..=5 {
            m.add_room((k, 0), &format!("road{}", k)).unwrap();
        }
        let p2 = m.path((0, 0), (5, 0)).unwrap();
        assert_eq!(p2.first(), Some(&(0, 0)));
        assert_eq!(p2.last(), Some(&(5, 0)));
        assert_eq!(p2.len(), 6);
        for w in p2.windows(2) {
            assert_eq!(m.distance(w[0], w[1]), Some(1));
        }
        // Path to itself.
        assert_eq!(m.path((0, 0), (0, 0)), Some(vec![(0, 0)]));
        // Missing endpoints.
        assert_eq!(m.path((9, 9), (0, 0)), None);
        assert_eq!(m.path((0, 0), (9, 9)), None);
    }

    #[test]
    fn map_temperature_reads_the_grid() {
        let mut m = HexRoomMap::new();
        assert_eq!(m.map_temperature(), None, "no readings yet");
        m.add_room((0, 0), "a").unwrap();
        m.add_room((1, 0), "b").unwrap();
        m.add_room((0, 1), "c").unwrap();
        m.add_room((1, 1), "unread").unwrap(); // no field — not part of the reading
                                               // Warm room: mood +0.5 -> warmth 0.075 (the elephant's neutral baseline
                                               // is -0.075 because cynicism 0.5 is subtracted raw)
        let warm = RoomField::new(0.5, 0.5, 0.5, 0.5, 0.0, 0.0, 0.5);
        let cold = RoomField::new(0.0, 0.5, 0.5, 1.0, 0.0, 0.0, 0.5);
        let neutral = RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, 0.0, 0.5);
        assert!(
            (neutral.warmth() - (-0.075)).abs() < 1e-12,
            "neutral baseline"
        );
        m.set_field((0, 0), warm).unwrap();
        m.set_field((1, 0), cold).unwrap();
        m.set_field((0, 1), neutral).unwrap();
        let t = m.map_temperature().unwrap();
        let expected = (warm.warmth() + cold.warmth() + neutral.warmth()) / 3.0;
        assert!(
            (t - expected).abs() < 1e-12,
            "temperature {} vs {}",
            t,
            expected
        );
        // map_panic reads too.
        assert_eq!(m.map_panic().unwrap(), 0.0);
        m.set_field((0, 0), RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, 0.9, 0.5))
            .unwrap();
        assert!((m.map_panic().unwrap() - 0.3).abs() < 1e-12);
    }

    #[test]
    fn deadband_rings_on_spreading_panic_and_stays_quiet_when_stable() {
        // A neutral room that is calm or panicking: warmth = -0.1 * panic.
        fn field(panic: f64) -> RoomField {
            RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, panic, 0.5)
        }

        let mut m = HexRoomMap::new();
        // The map: a tavern district around the origin, a temple district far
        // away, and an isolated far room.
        for c in m.region((0, 0), 1).unwrap() {
            m.add_room(c, &format!("tavern-{}{}", c.0, c.1)).unwrap();
        }
        for c in m.region((6, 0), 1).unwrap() {
            m.add_room(c, &format!("temple-{}{}", c.0, c.1)).unwrap();
        }
        m.add_room((12, 0), "lonely-hermit").unwrap();

        // Stable map: everyone calm. The deadband stays quiet no matter how
        // loud the caller claims the aggregate is below the threshold.
        for c in m.region((0, 0), 1).unwrap() {
            m.set_field(c, field(0.1)).unwrap();
        }
        for c in m.region((6, 0), 1).unwrap() {
            m.set_field(c, field(0.05)).unwrap();
        }
        m.set_field((12, 0), field(0.0)).unwrap();
        assert_eq!(m.deadband_ring(0.05, 0.5), None, "stable map stays quiet");
        assert_eq!(
            m.deadband_ring(0.49, 0.5),
            None,
            "below the band stays quiet"
        );

        // Panic spreads through the tavern district: the Tap and its six
        // neighbors all cross the threshold. The temple stays calm.
        let hot = m.region((0, 0), 1).unwrap(); // 7 rooms
        for c in &hot {
            m.set_field(*c, field(0.9)).unwrap();
        }
        let ring = m
            .deadband_ring(0.8, 0.5)
            .expect("the deadband must ring when the field crosses");
        assert!(ring.map_field >= ring.threshold);
        assert_eq!(ring.coords.len(), 7, "ring names the whole hot district");
        assert!(ring.region.iter().all(|n| n.starts_with("tavern-")));
        // The hermit alone at 0.95 panic is a smaller region; with two hot
        // clusters the ring must name the larger one.
        m.set_field((12, 0), field(0.95)).unwrap();
        let ring2 = m.deadband_ring(0.85, 0.5).unwrap();
        assert_eq!(ring2.coords.len(), 7, "larger region rings, not the hermit");
        assert!(ring2.region.iter().all(|n| n.starts_with("tavern-")));

        // When the aggregate crosses but NO single room does (whole map
        // warming gently past the deadband), the ring names every read room.
        let mut m2 = HexRoomMap::new();
        for k in 0..3 {
            m2.add_room((k, 0), &format!("warm-{}", k)).unwrap();
            m2.set_field((k, 0), field(0.1)).unwrap();
        }
        let ring3 = m2.deadband_ring(0.6, 0.5).unwrap();
        assert_eq!(ring3.coords.len(), 3);
    }

    #[test]
    fn front_direction_names_the_d6_unit_of_travel() {
        // Seeded at the origin, spreads east: the front is the unit 1.
        assert_eq!(front_direction(&[(0, 0)], &[(0, 0), (1, 0)]), Some((1, 0)));
        // Spreads north-east: the front is ω.
        assert_eq!(front_direction(&[(0, 0)], &[(0, 0), (0, 1)]), Some((0, 1)));
        // Reflections: west, and south-west (ω²).
        assert_eq!(front_direction(&[(1, 0)], &[(1, 0), (0, 0)]), Some((-1, 0)));
        assert_eq!(
            front_direction(&[(0, 0)], &[(0, 0), (-1, -1)]),
            Some((-1, -1))
        );
        // A longer march stays east; the turn north-east picks the unit 1+ω
        // (displacement (3,3) — exactly along the (1,1) direction).
        assert_eq!(
            front_direction(&[(0, 0), (1, 0)], &[(0, 0), (1, 0), (2, 0)]),
            Some((1, 0))
        );
        assert_eq!(
            front_direction(&[(0, 0), (1, 0), (2, 0)], &[(0, 0), (1, 0), (2, 0), (2, 1)]),
            Some((1, 1))
        );
        // Settled: no displacement, no front — a standing fire is not a montage.
        assert_eq!(front_direction(&[(0, 0), (1, 0)], &[(0, 0), (1, 0)]), None);
        // Empty frames carry no direction.
        assert_eq!(front_direction(&[], &[(0, 0)]), None);
        assert_eq!(front_direction(&[(0, 0)], &[]), None);
        // The front is always one of the six units — the argmax runs over
        // exactly that set (this displacement ties east/SE and breaks to
        // the first in directions order).
        let f = front_direction(&[(5, -3)], &[(5, -3), (7, -2)]).unwrap();
        assert!(hex_directions().contains(&f));
    }

    #[test]
    fn fields_read_back_what_set_field_wrote() {
        let mut m = HexRoomMap::new();
        m.add_room((0, 0), "read").unwrap();
        m.add_room((1, 0), "unread").unwrap();
        assert_eq!(m.fields().count(), 0, "nothing read yet");
        let warm = RoomField::new(0.5, 0.5, 0.5, 0.5, 0.0, 0.0, 0.5);
        m.set_field((0, 0), warm).unwrap();
        let collected: Vec<_> = m.fields().collect();
        assert_eq!(
            collected,
            vec![((0, 0), &warm)],
            "coord order, unread skipped"
        );
        // Overwriting a field replaces the reading.
        let hot = RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, 0.9, 0.5);
        m.set_field((0, 0), hot).unwrap();
        assert_eq!(m.fields().next().map(|(_, f)| *f), Some(hot));
    }

    #[test]
    fn ring_is_propagation_aware_the_montage_has_a_front() {
        fn field(panic: f64) -> RoomField {
            RoomField::new(0.0, 0.5, 0.5, 0.5, 0.0, panic, 0.5)
        }

        // A calm town; a fight will migrate through it hex-by-hex.
        let mut m = HexRoomMap::new();
        for c in m.region((0, 0), 3).unwrap() {
            m.add_room(c, &format!("town-{}{}", c.0, c.1)).unwrap();
            m.set_field(c, field(0.05)).unwrap();
        }

        // Frame 1: the fight is seeded at one hex. The ring names the seed;
        // the first frame of a montage has no front.
        m.set_field((0, 0), field(0.9)).unwrap();
        let r1 = m
            .deadband_ring(0.9, 0.5)
            .expect("the seed crosses the band");
        assert_eq!(r1.coords, vec![(0, 0)]);
        assert_eq!(r1.region, vec!["town-00"]);
        assert_eq!(
            r1.front, None,
            "a fresh blaze has no history to move against"
        );

        // Frame 2: it propagates to the east neighbor. The ring names the
        // connected region AND the front: the D₆ unit 1 (east).
        m.set_field((1, 0), field(0.9)).unwrap();
        let r2 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r2.coords, vec![(0, 0), (1, 0)], "the connected region");
        assert_eq!(r2.region, vec!["town-00", "town-10"]);
        assert_eq!(r2.front, Some((1, 0)), "the front is east");

        // Frame 3: it keeps moving east.
        m.set_field((2, 0), field(0.9)).unwrap();
        let r3 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r3.coords.len(), 3);
        assert!(r3.coords.contains(&(2, 0)));
        assert_eq!(r3.front, Some((1, 0)), "still heading east");

        // Frame 4: the fight turns north-east (hex (2,1)): front = 1+ω.
        m.set_field((2, 1), field(0.9)).unwrap();
        let r4 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r4.coords.len(), 4);
        assert!(r4.coords.contains(&(2, 1)));
        assert_eq!(r4.front, Some((1, 1)), "the front turns with the fight");

        // Frame 5: nothing moves. A standing fire is not a montage.
        let r5 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r5.coords.len(), 4);
        assert_eq!(r5.front, None, "settled: no direction of travel");

        // The band goes quiet — the fires die down — and the montage memory
        // resets, so a new seed starts its own sequence: frontless.
        assert!(m.deadband_ring(0.4, 0.5).is_none());
        for c in [(0, 0), (1, 0), (2, 0), (2, 1)] {
            m.set_field(c, field(0.05)).unwrap();
        }
        assert!(m.deadband_ring(0.4, 0.5).is_none(), "the town calms down");
        m.set_field((-3, 0), field(0.9)).unwrap();
        let r6 = m.deadband_ring(0.9, 0.5).unwrap();
        assert_eq!(r6.coords, vec![(-3, 0)]);
        assert_eq!(r6.front, None, "a new montage starts frontless");
    }

    #[test]
    fn guards_reject_impossible_coordinates() {
        let mut m = HexRoomMap::new();
        // Beyond the i32 E12 lattice: impossible.
        assert_eq!(
            m.add_room((1i64 << 40, 0), "far"),
            Err(MapError::OutOfLatticeRange)
        );
        assert_eq!(
            m.add_room((0, -(1i64 << 40)), "far"),
            Err(MapError::OutOfLatticeRange)
        );
        // Empty names are not rooms.
        assert_eq!(m.add_room((0, 0), ""), Err(MapError::UnnamedRoom));
        // Reading a hex that isn't a room.
        assert_eq!(
            m.set_field((0, 0), RoomField::default()),
            Err(MapError::RoomNotFound)
        );
        // Valid room, then guards that depend on its presence.
        m.add_room((0, 0), "tap").unwrap();
        assert_eq!(m.get((0, 0)), Some("tap"));
        assert!(m.contains((0, 0)));
        // Adding over an existing coord overwrites the name.
        m.add_room((0, 0), "the-tap").unwrap();
        assert_eq!(m.get((0, 0)), Some("the-tap"));
        // A path across the i64/hex lattice with endpoints that exist is fine,
        // but a missing endpoint is None.
        m.add_room((1, 0), "docks").unwrap();
        assert!(m.path((0, 0), (1, 0)).is_some());
        assert!(m.path((0, 0), (99, 99)).is_none());
    }
}
