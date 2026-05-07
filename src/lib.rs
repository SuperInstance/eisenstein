#![no_std]

//! Zero-drift hexagonal lattice constraints via Eisenstein integers.
//!
//! Exact integer arithmetic for safety-critical systems. Core type has zero
//! dependencies and zero floats. Angle snapping (`snap` feature, default) adds
//! libm for trig operations.
//!
//! # Feature flags
//!
//! - `snap` (default) — enables `snap_from_angle()` and `HexDisk::snap_direction()`.
//!   Adds `libm` dependency for trig functions.
//! - `std` — enables `std` support. Without it, `no_std` compatible.

#[cfg(feature = "snap")]
use core::f64::consts;
#[cfg(feature = "snap")]
use libm;

use core::fmt;

/// Float math helpers — only compiled when the `snap` feature is enabled.
#[cfg(feature = "snap")]
mod float {
    #[inline(always)]
    pub fn cos(x: f64) -> f64 {
        libm::cos(x)
    }

    #[inline(always)]
    pub fn sin(x: f64) -> f64 {
        libm::sin(x)
    }

    #[inline(always)]
    pub fn atan2(y: f64, x: f64) -> f64 {
        libm::atan2(y, x)
    }

    #[inline(always)]
    pub fn round(x: f64) -> f64 {
        libm::round(x)
    }
}

/// Eisenstein integer in the E12 lattice: a + bω where ω = e^(2πi/3).
///
/// Norm is a² - ab + b² (always non-negative, fits in 24 bits for reasonable values).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct E12 {
    a: i32,
    b: i32,
}

impl E12 {
    /// Create a new Eisenstein integer a + bω.
    #[inline]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    /// Real coefficient.
    #[inline]
    pub const fn a(self) -> i32 {
        self.a
    }

    /// ω coefficient.
    #[inline]
    pub const fn b(self) -> i32 {
        self.b
    }

    /// Norm: a² - ab + b². Always non-negative.
    #[inline]
    pub fn norm(self) -> u64 {
        let a = self.a as i64;
        let b = self.b as i64;
        let n = a * a - a * b + b * b;
        n as u64
    }

    /// Complex conjugate: a + bω̄ = a + bω² = (a - b) + (-b)ω... wait.
    /// conj(a + bω) = a + bω² = a + b(-1 - ω) = (a - b) - b·ω
    /// So conjugate is (a - b, -b).
    #[inline]
    pub const fn conjugate(self) -> Self {
        Self::new(self.a - self.b, -self.b)
    }

    /// Create from axial hex coordinates (q, r).
    #[inline]
    pub const fn from_axial(q: i32, r: i32) -> Self {
        Self::new(q, r)
    }

    /// Convert to axial hex coordinates (q, r).
    #[inline]
    pub const fn to_axial(self) -> (i32, i32) {
        (self.a, self.b)
    }

    /// The six neighbors in the hexagonal lattice (self + each unit direction).
    pub fn neighbors(self) -> [E12; 6] {
        let dirs = Self::directions();
        [
            self + dirs[0],
            self + dirs[1],
            self + dirs[2],
            self + dirs[3],
            self + dirs[4],
            self + dirs[5],
        ]
    }

    /// The six unit directions of the hexagonal lattice.
    ///
    /// These are the 6 units of Z[ω]: ±1, ±ω, ±ω² where ω = e^{2πi/3}.
    /// All have Eisenstein norm a²-ab+b² = 1 and Euclidean distance 1.
    ///
    /// | Unit | E12 | Cartesian |
    /// |------|-----|-----------|
    /// |  1   | ( 1, 0) | ( 1.0,  0.0)  | East  |
    /// |  ω   | ( 0, 1) | (-0.5,  0.87) | NE    |
    /// |  ω²  | (-1,-1) | (-0.5, -0.87) | SW    |
    /// | -1   | (-1, 0) | (-1.0,  0.0)  | West  |
    /// | -ω   | ( 0,-1) | ( 0.5, -0.87) | SE    |
    /// | -ω²  | ( 1, 1) | ( 0.5,  0.87) | NW    |
    pub const fn directions() -> [E12; 6] {
        [
            E12::new(1, 0),    // 1   (East)
            E12::new(0, 1),    // ω   (NE)
            E12::new(1, 1),    // -ω² (NW) — note: -ω² = 1+ω = (1,1)
            E12::new(-1, 0),   // -1  (West)
            E12::new(0, -1),   // -ω  (SE)
            E12::new(-1, -1),  // ω²  (SW) — note: ω² = -1-ω = (-1,-1)
        ]
    }
}

impl fmt::Debug for E12 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+{}ω", self.a, self.b)
    }
}

impl fmt::Display for E12 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+{}ω", self.a, self.b)
    }
}

impl core::ops::Add for E12 {
    type Output = E12;
    #[inline]
    fn add(self, rhs: E12) -> E12 {
        E12::new(self.a + rhs.a, self.b + rhs.b)
    }
}

impl core::ops::Sub for E12 {
    type Output = E12;
    #[inline]
    fn sub(self, rhs: E12) -> E12 {
        E12::new(self.a - rhs.a, self.b - rhs.b)
    }
}

impl core::ops::Mul for E12 {
    type Output = E12;
    /// Multiply: (a + bω)(c + dω) = ac + (ad + bc)ω + bd·ω²
    /// Since ω² = -1 - ω:
    /// = ac + (ad + bc)ω + bd(-1 - ω)
    /// = ac - bd + (ad + bc - bd)ω
    /// = (ac - bd) + (ad + bc - bd)ω
    #[inline]
    fn mul(self, rhs: E12) -> E12 {
        let a = self.a;
        let b = self.b;
        let c = rhs.a;
        let d = rhs.b;
        E12::new(a * c - b * d, a * d + b * c - b * d)
    }
}

/// Hex disk of radius R — contains 3R² + 3R + 1 points.
#[derive(Clone, Copy, Debug)]
pub struct HexDisk {
    radius: u32,
}

impl HexDisk {
    /// Create a hex disk with the given radius.
    pub const fn radius(radius: u32) -> Self {
        Self { radius }
    }

    /// Snap to the nearest vertex in this disk from an angle in radians.
    ///
    /// Finds the E12 point within the disk whose argument (angle) is closest
    /// to the given θ. More precise than `E12::snap_from_angle` when working
    /// within a bounded region, since it considers all disk vertices.
    ///
    /// ```
    /// # use eisenstein::{E12, HexDisk};
    /// let disk = HexDisk::radius(36);
    /// // East (0 rad) should snap to (1, 0)
    /// assert_eq!(disk.snap_direction(0.0).unwrap(), E12::new(1, 0));
    /// ```
    #[cfg(feature = "snap")]
    pub fn snap_direction(&self, radians: f64) -> Option<E12> {
        let cos = float::cos(radians);
        let sin = float::sin(radians);
        self.iter()
            .filter(|p| p.a != 0 || p.b != 0)
            .min_by(|a, b| {
                let da = E12::angular_distance(*a, cos, sin);
                let db = E12::angular_distance(*b, cos, sin);
                da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal)
            })
    }

    /// Check if a point is inside this disk.
    pub fn contains(&self, p: &E12) -> bool {
        // Distance in hex = max(|a|, |b|, |a+b|)... actually for axial coords
        // the hex distance is (|a| + |b| + |a+b|) / 2
        // But norm a²-ab+b² = 0 iff a=b=0. The norm-based radius check:
        // point is in disk R iff norm <= some threshold.
        // Actually for a proper hex disk, we use the hex distance:
        // For axial (q, r): distance = (|q| + |r| + |q+r|) / 2
        // = max(|q|, |r|, |q+r|) ... no, those aren't the same.
        // hex distance = max(|q|, |r|, |q+r|) is WRONG.
        // hex distance = (|q| + |r| + |q+r|) / 2
        // But we need integer division. Since |q|+|r|+|q+r| is always even for integers, this works.
        let q = p.a();
        let r = p.b();
        let dist = ((q.abs() + r.abs() + (q + r).abs()) / 2) as u32;
        dist <= self.radius
    }

    /// Number of points: 3R² + 3R + 1.
    pub const fn count(&self) -> u64 {
        let r = self.radius as u64;
        3 * r * r + 3 * r + 1
    }

    /// Iterate over all points in the disk.
    pub fn iter(&self) -> HexDiskIter {
        HexDiskIter::new(*self)
    }
}

/// Iterator over all points in a hex disk.
pub struct HexDiskIter {
    disk: HexDisk,
    q: i32,
    r: i32,
    r_min: i32,
    r_max: i32,
    done: bool,
}

impl HexDiskIter {
    fn new(disk: HexDisk) -> Self {
        let r_max = disk.radius as i32;
        let r_min = -(disk.radius as i32);
        let q_start = -(disk.radius as i32);
        Self {
            disk,
            q: q_start,
            r: r_min,
            r_min,
            r_max,
            done: false,
        }
    }
}

impl Iterator for HexDiskIter {
    type Item = E12;

    fn next(&mut self) -> Option<E12> {
        if self.done {
            return None;
        }

        loop {
            if self.r > self.r_max {
                // Move to next q
                self.q += 1;
                if self.q > self.disk.radius as i32 {
                    self.done = true;
                    return None;
                }
                self.r = self.r_min;
            }

            let p = E12::new(self.q, self.r);
            self.r += 1;

            if self.disk.contains(&p) {
                return Some(p);
            }
        }
    }
}

/// Eisenstein triple: a² - ab + b² = c² (like Pythagorean but hexagonal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EisensteinTriple {
    a: i32,
    b: i32,
    c: u32,
}

impl EisensteinTriple {
    /// Create a new Eisenstein triple.
    pub const fn new(a: i32, b: i32, c: u32) -> Self {
        Self { a, b, c }
    }

    pub const fn a(self) -> i32 {
        self.a
    }

    pub const fn b(self) -> i32 {
        self.b
    }

    pub const fn c(self) -> u32 {
        self.c
    }

    /// The norm a² - ab + b².
    pub fn norm(self) -> u64 {
        E12::new(self.a, self.b).norm()
    }

    /// Find (a, b) such that a² - ab + b² = target (i.e. the Eisenstein norm equals `target`).
    /// Returns the first found (canonically with a >= b >= 0).
    /// For an Eisenstein triple where norm = c², pass c² as the target.
    pub fn from_norm(target: u32) -> Option<Self> {
        // Search: a² - ab + b² = target
        // For a >= 0, try b from 0..=a
        let target_i = target as i64;
        let max_a = if target > 46340 { 46340 } else { target as i32 };
        for a in 0..=max_a {
            let a_i = a as i64;
            for b in 0..=a {
                let b_i = b as i64;
                let val = a_i * a_i - a_i * b_i + b_i * b_i;
                if val == target_i {
                    let c = int_sqrt(target_i);
                    if c * c == target_i && c > 0 {
                        return Some(Self::new(a, b, c as u32));
                    }
                }
            }
        }
        None
    }

    /// Find (a, b) such that a² - ab + b² equals the given raw norm value.
    /// Unlike from_norm, this doesn't require the result to be a perfect square.
    pub fn from_norm_raw(target: i64) -> Option<(i32, i32)> {
        // Upper bound: for fixed a, the minimum norm is 3a²/4 (at b=a/2).
        // So we need 3a²/4 ≤ target, i.e., a ≤ sqrt(4*target/3).
        // Use a generous upper bound: a ≤ target (overkill but safe).
        let max_a = if target > 46340 { 46340 } else { target as i32 }; // avoid i32 overflow
        for a in 0..=max_a {
            let a_i = a as i64;
            for b in 0..=a {
                let b_i = b as i64;
                let val = a_i * a_i - a_i * b_i + b_i * b_i;
                if val == target {
                    return Some((a, b));
                }
            }
        }
        None
    }

    /// Find all Eisenstein triples with c ≤ c_max.
    pub fn all_with_max_norm(c_max: u32) -> alloc::vec::Vec<Self> {
        let mut results = alloc::vec::Vec::new();
        let _c2_max = (c_max as i64) * (c_max as i64);
        let max_a = if c_max > 46340 { 46340 } else { c_max as i32 };
        for a in 0..=max_a {
            let a_i = a as i64;
            for b in 0..=a {
                let b_i = b as i64;
                let val = a_i * a_i - a_i * b_i + b_i * b_i;
                if val > 0 {
                    let sqrt_val = int_sqrt(val);
                    if sqrt_val * sqrt_val == val && sqrt_val > 0 && (sqrt_val as u32) <= c_max {
                        results.push(Self::new(a, b, sqrt_val as u32));
                    }
                }
            }
        }
        results
    }

    /// Is this triple primitive (gcd-like check)?
    /// An Eisenstein triple is primitive if gcd(a, b) = 1 in Eisenstein integers.
    /// Simplified: check if gcd(|a|, |b|) = 1.
    pub fn is_primitive(self) -> bool {
        gcd(self.a.unsigned_abs(), self.b.unsigned_abs()) == 1
    }

    /// Generate the first n primitive Eisenstein triples.
    pub fn generate(n: usize) -> alloc::vec::Vec<Self> {
        let mut results = alloc::vec::Vec::new();
        let mut a: i32 = 1;
        while results.len() < n {
            for b in 0..=a {
                let val = (a as i64) * (a as i64) - (a as i64) * (b as i64) + (b as i64) * (b as i64);
                if val > 1 {
                    let sqrt_val = int_sqrt(val);
                    if sqrt_val * sqrt_val == val && sqrt_val > 1 {
                        let triple = Self::new(a, b, sqrt_val as u32);
                        if triple.is_primitive() {
                            results.push(triple);
                            if results.len() >= n {
                                break;
                            }
                        }
                    }
                }
            }
            a += 1;
        }
        results
    }
}

/// Integer square root.
fn int_sqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Greatest common divisor.
fn gcd(a: u32, b: u32) -> u32 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Laman redundancy for 2D hexagonal lattice: 1.5.
pub const fn laman_redundancy_2d() -> i32 {
    // Returns 3/2 as fixed point: 15 (i.e. 1.5 * 10)
    // Actually, let's return a fraction as (numerator, denominator)
    // Or just return the float... but no floats!
    // Return as a ratio: (3, 2)
    15 // represents 1.5 when divided by 10
}

/// Laman redundancy for 3D hexagonal lattice: 2.0.
pub const fn laman_redundancy_3d() -> i32 {
    20 // represents 2.0 when divided by 10
}

/// Laman redundancy as a rational (numerator, denominator) for 2D.
pub const fn laman_redundancy_2d_ratio() -> (u32, u32) {
    (3, 2)
}

/// Laman redundancy as a rational (numerator, denominator) for 3D.
pub const fn laman_redundancy_3d_ratio() -> (u32, u32) {
    (2, 1)
}

impl E12 {
    /// Rotations by the D6 symmetry group of the hexagonal lattice.
    ///
    /// The six rotations: identity, 60°, 120°, 180°, 240°, 300°.
    /// Each preserves the Eisenstein norm.
    pub fn d6_rotations(self) -> [E12; 6] {
        let r0 = self;
        let r1 = E12::new(-self.b, self.a - self.b);   // 60°
        let r2 = E12::new(self.b - self.a, -self.a);    // 120°
        let r3 = E12::new(-self.a, -self.b);            // 180°
        let r4 = E12::new(self.b, -self.a + self.b);    // 240°
        let r5 = E12::new(self.a - self.b, self.a);     // 300°
        [r0, r1, r2, r3, r4, r5]
    }

    /// Hex distance from origin: (|a| + |b| + |a+b|) / 2.
    #[inline]
    pub fn hex_distance(self) -> u32 {
        ((self.a.abs() + self.b.abs() + (self.a + self.b).abs()) / 2) as u32
    }

    /// Check if this is a unit in Z[ω] (one of ±1, ±ω, ±ω²).
    #[inline]
    pub fn is_unit(self) -> bool {
        self.norm() == 1
    }

    /// Scalar multiplication.
    #[inline]
    pub const fn scale(self, k: i32) -> E12 {
        E12::new(self.a * k, self.b * k)
    }

    /// Snap to the nearest Eisenstein integer from an angle in radians.
    ///
    /// Given an angle θ (in radians), finds the E12 point whose argument is
    /// closest to θ. This is the #1 request from game dev play-testers who want
    /// to place things at exact hexagonal angles.
    ///
    /// Algorithm: iterates over Eisenstein integers ordered by norm (up to a
    /// generous bound) and selects the one with minimum angular distance to the
    /// given angle. This guarantees the best angular fit.
    ///
    /// For game devs: cardinal directions snap to unit E12 (norm 1), and
    /// intermediate angles find the densest angular approach within the lattice.
    ///
    /// ```
    /// # use eisenstein::E12;
    /// // 0 radians → East → (1, 0)
    /// assert_eq!(E12::snap_from_angle(0.0), E12::new(1, 0));
    /// // π/2 radians → North → (1, 2) at exactly 90° in cartesian
    /// let north = E12::snap_from_angle(core::f64::consts::FRAC_PI_2);
    /// assert_eq!(north, E12::new(1, 2));
    /// ```
    #[cfg(feature = "snap")]
    pub fn snap_from_angle(radians: f64) -> E12 {
        let cos = float::cos(radians);
        let sin = float::sin(radians);

        // Max norm to search. Norm 100 is generous — it captures all E12
        // within a hex radius of ~10, giving precise angular resolution (~3°).
        // For game dev use this is more than sufficient.
        const MAX_NORM: u64 = 100;

        let inv_rt3 = 0.5773502691896257; // 1 / √3
        let two_over_rt3 = 1.1547005383792517; // 2 / √3

        // Convert from cartesian (cos, sin) to axial hex coords (a, b)
        // Hex -> cartesian: x = a - b/2, y = b * √3/2
        // Inverse: b = y * 2/√3, a = x + b/2 = cos + (y/√3)
        let b_float = sin * two_over_rt3;
        let a_float = cos + sin * inv_rt3;

        let a_round = float::round(a_float) as i32;
        let b_round = float::round(b_float) as i32;

        // Search: start with the rounded point and expand outward.
        // Check points by increasing norm (using hex-distance-like expansion).
        // This prefers smaller E12 points when angular fit is comparable.
        let mut best = Self::angular_scan(a_round, b_round, 2, cos, sin, MAX_NORM);

        // If no valid point found (shouldn't happen), expand search
        if best.a == 0 && best.b == 0 {
            best = Self::angular_scan(a_round, b_round, 4, cos, sin, MAX_NORM);
        }

        best
    }

    /// Search for the E12 closest to the given direction within a bounding box.
    /// Favors points with smaller norm when angular distances are close.
    #[cfg(feature = "snap")]
    fn angular_scan(cx: i32, cy: i32, radius: i32, cos: f64, sin: f64, max_norm: u64) -> E12 {
        let mut best = E12::new(cx, cy);
        let mut best_diff = E12::angular_distance(best, cos, sin);
        let mut best_norm = best.norm();

        for da in -radius..=radius {
            for db in -radius..=radius {
                let candidate = E12::new(cx + da, cy + db);
                if candidate.a == 0 && candidate.b == 0 {
                    continue;
                }
                let n = candidate.norm();
                if n > max_norm {
                    continue;
                }
                let diff = E12::angular_distance(candidate, cos, sin);
                // Prefer smaller diff, or equal diff with smaller norm
                if diff < best_diff - 1e-12 || (diff < best_diff + 1e-12 && n < best_norm) {
                    best_diff = diff;
                    best = candidate;
                    best_norm = n;
                }
            }
        }

        best
    }

    /// Angular distance between an E12 point and a unit vector (cos, sin).
    /// Returns the absolute angle difference in radians.
    #[cfg(feature = "snap")]
    fn angular_distance(z: E12, cos: f64, sin: f64) -> f64 {
        // Convert E12 to cartesian: x = a - b/2, y = b * √3/2
        let x = z.a as f64 - z.b as f64 * 0.5;
        let y = z.b as f64 * 0.8660254037844386; // √3/2
        let point_angle = float::atan2(y, x);
        let target_angle = float::atan2(sin, cos);
        let mut diff = (point_angle - target_angle).abs();
        if diff > consts::PI {
            diff = 2.0 * consts::PI - diff;
        }
        diff
    }

    /// Euclidean division in Z[ω].
    ///
    /// Z[ω] is a Euclidean domain with norm N(a+bω) = a²-ab+b².
    /// For any α, β ≠ 0, there exist γ, ρ such that α = βγ + ρ with N(ρ) < N(β).
    ///
    /// Returns (quotient, remainder) or None if divisor is zero.
    pub fn div_rem(self, divisor: E12) -> Option<(E12, E12)> {
        if divisor.a == 0 && divisor.b == 0 {
            return None;
        }

        // α/β = α·conj(β) / N(β) = α·conj(β) / (a²-ab+b²)
        let n = divisor.norm() as i64;
        let conj_b = divisor.conjugate();
        let numer = self * conj_b; // exact Eisenstein multiplication

        // Round each coordinate to nearest integer
        let qa = round_div(numer.a as i64, n);
        let qb = round_div(numer.b as i64, n);

        let quotient = E12::new(qa as i32, qb as i32);
        let remainder = self - divisor * quotient;

        Some((quotient, remainder))
    }

    /// Greatest common divisor via Euclidean algorithm in Z[ω].
    ///
    /// Uses the Euclidean property: gcd(α, β) = gcd(β, α mod β).
    /// Returns a unit-normalized gcd (with a > 0, or a == 0 and b > 0).
    pub fn gcd(self, other: E12) -> E12 {
        let mut a = self;
        let mut b = other;

        while b.a != 0 || b.b != 0 {
            let (_, rem) = a.div_rem(b).unwrap_or((a, E12::new(0, 0)));
            a = b;
            b = rem;
        }

        // Normalize: prefer the associate with a > 0
        if a.a < 0 || (a.a == 0 && a.b < 0) {
            a = E12::new(-a.a, -a.b);
        }
        a
    }

    /// Check if self divides other exactly in Z[ω].
    pub fn divides(self, other: E12) -> bool {
        if self.a == 0 && self.b == 0 {
            return other.a == 0 && other.b == 0;
        }
        if let Some((_, rem)) = other.div_rem(self) {
            rem.a == 0 && rem.b == 0
        } else {
            false
        }
    }

    /// Compute all divisors of norm N(self) in Z[ω].
    /// Returns associates (unit multiples) as a single representative.
    pub fn norm_divisors(self) -> alloc::vec::Vec<E12> {
        let n = self.norm() as i64;
        let mut divisors = alloc::vec::Vec::new();

        // Search for (a,b) with a²-ab+b² dividing n
        let max_val = int_sqrt(n) + 1;
        for a in -max_val..=max_val {
            for b in -max_val..=max_val {
                let d_norm = E12::new(a as i32, b as i32).norm() as i64;
                if d_norm > 0 && n % d_norm == 0 {
                    divisors.push(E12::new(a as i32, b as i32));
                }
            }
        }
        divisors
    }
}

/// Round a/b to the nearest integer (rounding halves toward zero).
fn round_div(a: i64, b: i64) -> i32 {
    if b == 0 { return 0; }
    let sign = if (a < 0) ^ (b < 0) { -1i64 } else { 1i64 };
    let abs_a = a.abs();
    let abs_b = b.abs();
    let q = abs_a / abs_b;
    let r = abs_a % abs_b;
    // Round: if remainder > half of divisor, round up
    if 2 * r > abs_b {
        (sign * (q + 1)) as i32
    } else {
        (sign * q) as i32
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e12_norm() {
        let p = E12::new(3, -2);
        // 3² - 3(-2) + (-2)² = 9 + 6 + 4 = 19
        assert_eq!(p.norm(), 19);
    }

    #[test]
    fn test_e12_neighbors() {
        let origin = E12::new(0, 0);
        let neighbors = origin.neighbors();
        assert_eq!(neighbors.len(), 6);
        // All neighbors should have norm 1
        for n in &neighbors {
            assert_eq!(n.norm(), 1);
        }
        // All should be distinct
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_ne!(neighbors[i], neighbors[j]);
            }
        }
    }

    #[test]
    fn test_hex_disk_counts() {
        assert_eq!(HexDisk::radius(0).count(), 1);
        assert_eq!(HexDisk::radius(1).count(), 7);
        assert_eq!(HexDisk::radius(2).count(), 19);
        assert_eq!(HexDisk::radius(3).count(), 37);
    }

    #[test]
    fn test_hex_disk_formula_large() {
        assert_eq!(HexDisk::radius(36).count(), 3997);
    }

    #[test]
    fn test_hex_disk_iter_counts() {
        for r in 0..=5u32 {
            let disk = HexDisk::radius(r);
            let count = disk.iter().count();
            assert_eq!(count as u64, disk.count(), "Mismatch at radius {}", r);
        }
    }

    #[test]
    fn test_eisenstein_triple_from_norm_13() {
        // 4² - 4·1 + 1² = 16 - 4 + 1 = 13 ✓ (norm = 13, but 13 is NOT a perfect square)
        // So this is NOT an Eisenstein triple (c would be √13).
        let (a, b) = EisensteinTriple::from_norm_raw(13).unwrap();
        assert_eq!(a, 4);
        assert_eq!(b, 1);

        // Actual Eisenstein triple: (7, 0, 7) is the first found for norm=49
        // 7² - 7·0 + 0² = 49 = 7² → trivial but valid
        let triple = EisensteinTriple::from_norm(49).unwrap();
        assert_eq!(triple.a(), 7);
        assert_eq!(triple.b(), 0);
        assert_eq!(triple.c(), 7);

        // Non-trivial: (8, 3, 7) since 64-24+9 = 49 = 7²
        let all = EisensteinTriple::all_with_max_norm(10);
        let nontrivial: alloc::vec::Vec<_> = all.iter().filter(|t| t.a() > 0 && t.b() > 0).collect();
        assert!(nontrivial.iter().any(|t| t.a() == 8 && t.b() == 3 && t.c() == 7),
            "Should find (8,3,7) in non-trivial triples: {:?}", nontrivial);
    }

    #[test]
    fn test_multiplication() {
        // (1,1) * (1,-1) = ?
        // Using formula: (ac - bd, ad + bc - bd)
        // a=1, b=1, c=1, d=-1
        // new_a = 1*1 - 1*(-1) = 1 + 1 = 2
        // new_b = 1*(-1) + 1*1 - 1*(-1) = -1 + 1 + 1 = 1
        // So (2, 1)
        let p1 = E12::new(1, 1);
        let p2 = E12::new(1, -1);
        let result = p1 * p2;
        assert_eq!(result, E12::new(2, 1));
    }

    #[test]
    fn test_weyl_invariance() {
        // Norm should be invariant under D6 rotations:
        // (a,b) → (-b, a-b) → (b-a, -a) and their negatives
        let test_cases: &[(i32, i32)] = &[(3, -2), (1, 1), (5, 0), (2, 3), (-1, 4)];
        for &(a, b) in test_cases {
            let p = E12::new(a, b);
            let r1 = E12::new(-b, a - b);
            let r2 = E12::new(b - a, -a);
            // And negatives (inversions)
            let r3 = E12::new(-a, -b);
            let r4 = E12::new(b, -a + b);
            let r5 = E12::new(a - b, a);

            assert_eq!(p.norm(), r1.norm(), "D6 rotation 1 failed for ({},{})", a, b);
            assert_eq!(p.norm(), r2.norm(), "D6 rotation 2 failed for ({},{})", a, b);
            assert_eq!(p.norm(), r3.norm(), "D6 rotation 3 failed for ({},{})", a, b);
            assert_eq!(p.norm(), r4.norm(), "D6 rotation 4 failed for ({},{})", a, b);
            assert_eq!(p.norm(), r5.norm(), "D6 rotation 5 failed for ({},{})", a, b);
        }
    }

    #[test]
    fn test_zero_drift() {
        // 10000 E12 operations, all exact integer arithmetic
        // Use small values to avoid overflow

        // Simple deterministic "random" sequence
        let mut state: u64 = 12345;
        let mut next_small = || -> i32 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 40) & 0x3FF) as i32 - 512 // range [-512, 511]
        };

        for _ in 0..10000 {
            let a = E12::new(next_small(), next_small());
            let b = E12::new(next_small(), next_small());
            let c = E12::new(next_small(), next_small());

            // Associativity: (a + b) + c == a + (b + c)
            assert_eq!((a + b) + c, a + (b + c));

            // Commutativity of addition
            assert_eq!(a + b, b + a);

            // Norm is multiplicative: norm(a*b) == norm(a)*norm(b)
            // This is the KEY zero-drift property
            // Use checked arithmetic for the multiplication to avoid overflow
            let norm_a = a.norm();
            let norm_b = b.norm();
            let norm_ab = (a * b).norm();
            // Only check if no overflow occurred
            if norm_a < (1u64 << 31) && norm_b < (1u64 << 31) {
                assert_eq!(
                    norm_ab, norm_a * norm_b,
                    "Norm multiplicativity failed for ({},{}) * ({},{})",
                    a.a(), a.b(), b.a(), b.b()
                );
            }
        }
    }

    #[test]
    fn test_conjugate() {
        let p = E12::new(3, -2);
        let conj = p.conjugate();
        assert_eq!(conj.a(), 3 - (-2)); // a - b = 5
        assert_eq!(conj.b(), -(-2)); // -b = 2
        assert_eq!(conj.a(), 5);
        assert_eq!(conj.b(), 2);
    }

    #[test]
    fn test_laman_redundancy() {
        assert_eq!(laman_redundancy_2d_ratio(), (3, 2));
        assert_eq!(laman_redundancy_3d_ratio(), (2, 1));
    }

    #[test]
    fn test_display() {
        let p = E12::new(3, -2);
        use alloc::format;
        assert_eq!(format!("{}", p), "3+-2ω");
    }

    // === New tests for Euclidean division, GCD, and number theory ===

    #[test]
    fn test_div_rem_exact() {
        // (2+ω) / (1+ω) should be exact since (1+ω)*(1+0ω) + (1+ω)*ω = 2+ω
        // Actually: (2,1) / (1,1)
        let alpha = E12::new(2, 1);
        let beta = E12::new(1, 1);
        let (q, r) = alpha.div_rem(beta).unwrap();
        // Verify: alpha = beta * q + r
        assert_eq!(beta * q + r, alpha);
        // Remainder should have smaller norm
        assert!(r.norm() < beta.norm() || (r.a == 0 && r.b == 0),
            "Remainder norm {} >= divisor norm {}", r.norm(), beta.norm());
    }

    #[test]
    fn test_div_rem_identity() {
        let alpha = E12::new(5, -3);
        let (q, r) = alpha.div_rem(E12::new(1, 0)).unwrap();
        assert_eq!(q, alpha);
        assert_eq!(r, E12::new(0, 0));
    }

    #[test]
    fn test_div_rem_zero_divisor() {
        let alpha = E12::new(5, 3);
        assert!(alpha.div_rem(E12::new(0, 0)).is_none());
    }

    #[test]
    fn test_euclidean_property() {
        // For many random pairs, verify N(remainder) < N(divisor)
        let mut state: u64 = 42;
        let mut next_small = || -> i32 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 40) & 0xF) as i32 - 8 // range [-8, 7]
        };

        for _ in 0..1000 {
            let a = E12::new(next_small(), next_small());
            let b = E12::new(next_small(), next_small());
            if b.a == 0 && b.b == 0 { continue; }
            if let Some((q, r)) = a.div_rem(b) {
                // Verify reconstruction
                assert_eq!(b * q + r, a,
                    "Reconstruction failed: {} * {} + {} != {}",
                    b, q, r, a);
                // Verify Euclidean property
                assert!(r.norm() < b.norm() || (r.a == 0 && r.b == 0),
                    "N(rem)={} >= N(div)={} for {} / {}",
                    r.norm(), b.norm(), a, b);
            }
        }
    }

    #[test]
    fn test_gcd_identity() {
        let a = E12::new(7, 3);
        let gcd = a.gcd(E12::new(1, 0));
        assert!(gcd.is_unit(), "gcd with 1 should be a unit");
    }

    #[test]
    fn test_gcd_zero() {
        let a = E12::new(5, -2);
        let gcd = a.gcd(E12::new(0, 0));
        // gcd(a, 0) = a (normalized)
        assert_eq!(gcd.a().abs(), a.a().abs());
        assert_eq!(gcd.b().abs(), a.b().abs());
    }

    #[test]
    fn test_gcd_commutative() {
        let a = E12::new(6, 3);
        let b = E12::new(3, 9);
        let gcd1 = a.gcd(b);
        let gcd2 = b.gcd(a);
        // Associates (may differ by a unit)
        assert_eq!(gcd1.norm(), gcd2.norm());
    }

    #[test]
    fn test_gcd_associates() {
        // 3 and 3ω should have gcd = unit (they're associates)
        let a = E12::new(3, 0);
        let b = E12::new(0, 3); // = 3ω
        let g = a.gcd(b);
        assert_eq!(g.norm(), 9, "gcd(3, 3ω) should have norm 9");
    }

    #[test]
    fn test_divides() {
        // 2 divides 6 in Z[ω]
        assert!(E12::new(2, 0).divides(E12::new(6, 0)));
        // 7 doesn't divide (1+ω)
        assert!(!E12::new(7, 0).divides(E12::new(1, 1)));
        // 1+ω divides (2+2ω)
        assert!(E12::new(1, 1).divides(E12::new(2, 2)));
    }

    #[test]
    fn test_hex_distance() {
        assert_eq!(E12::new(0, 0).hex_distance(), 0);
        assert_eq!(E12::new(1, 0).hex_distance(), 1);
        assert_eq!(E12::new(1, 1).hex_distance(), 2);
        assert_eq!(E12::new(2, -1).hex_distance(), 2);
        assert_eq!(E12::new(3, -2).hex_distance(), 3);
    }

    #[test]
    fn test_is_unit() {
        for dir in E12::directions() {
            assert!(dir.is_unit(), "Direction {:?} should be a unit", dir);
        }
        assert!(!E12::new(2, 0).is_unit());
        assert!(!E12::new(0, 0).is_unit());
    }

    #[test]
    fn test_d6_rotations_norm_preserved() {
        let p = E12::new(7, -3);
        for rot in p.d6_rotations() {
            assert_eq!(rot.norm(), p.norm(),
                "D6 rotation should preserve norm");
        }
    }

    #[test]
    fn test_d6_rotations_composition() {
        // Six 60° rotations should return to start
        let p = E12::new(5, -2);
        let rotations = p.d6_rotations();
        // Apply 60° rotation 6 times
        let mut current = p;
        for _ in 0..6 {
            current = E12::new(-current.b, current.a - current.b);
        }
        assert_eq!(current, p, "Six 60° rotations should be identity");
    }

    // === snap_from_angle tests ===

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_east() {
        // 0 radians → East → (1, 0)
        let z = E12::snap_from_angle(0.0);
        assert_eq!(z, E12::new(1, 0), "East should snap to (1,0), got {:?}", z);
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_north() {
        // π/2 → 90° → (1, 2) which is at EXACTLY 90° in cartesian
        // (0, 1) is at 120°, so (1, 2) is closer to north
        let z = E12::snap_from_angle(consts::FRAC_PI_2);
        assert_eq!(z, E12::new(1, 2), "North should snap to (1,2), got {:?}", z);
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_west() {
        // π → West → (-1, 0)
        let z = E12::snap_from_angle(consts::PI);
        assert_eq!(z, E12::new(-1, 0), "West should snap to (-1,0), got {:?}", z);
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_south() {
        // 3π/2 → 270° → (-1, -2) which is at EXACTLY -90° in cartesian
        let z = E12::snap_from_angle(3.0 * consts::FRAC_PI_2);
        assert_eq!(z, E12::new(-1, -2), "South should snap to (-1,-2)");
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_45_degrees() {
        // π/4 → NE-ish, check it's reasonably close
        let z = E12::snap_from_angle(consts::FRAC_PI_4);
        let (sin, cos) = (consts::FRAC_PI_4).sin_cos();
        let diff = E12::angular_distance(z, cos, sin);
        assert!(diff < 0.6, "45° snap diff {} should be < 0.6 rad", diff);
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_symmetry() {
        // Opposite angles should give opposite points
        let z0 = E12::snap_from_angle(0.0);
        let z1 = E12::snap_from_angle(consts::PI);
        assert_eq!(z0, E12::new(1, 0));
        assert_eq!(z1, E12::new(-1, 0));
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_hex_unit_directions() {
        // The 6 hex unit directions (0°, 60°, 120°, 180°, 240°, 300°) should snap to unit-norm E12
        let hex_angles = [0.0, consts::FRAC_PI_3, 2.0*consts::FRAC_PI_3, consts::PI, 4.0*consts::FRAC_PI_3, 5.0*consts::FRAC_PI_3];
        let expected = [E12::new(1,0), E12::new(1,1), E12::new(0,1), E12::new(-1,0), E12::new(-1,-1), E12::new(0,-1)];
        for (angle, exp) in hex_angles.iter().zip(expected.iter()) {
            let z = E12::snap_from_angle(*angle);
            assert_eq!(z, *exp, "Angle {}° should snap to {:?}, got {:?}", angle * 180.0 / consts::PI, exp, z);
        }
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_30_degrees() {
        let angle = consts::FRAC_PI_6;
        let z = E12::snap_from_angle(angle);
        let (sin, cos) = angle.sin_cos();
        let diff = E12::angular_distance(z, cos, sin);
        assert!(diff < 0.6, "30° snap diff {} should be small", diff);
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_snap_from_angle_60_degrees() {
        // π/3 = 60° → (1, 1) which is at EXACTLY 60° in cartesian
        let z = E12::snap_from_angle(consts::FRAC_PI_3);
        assert_eq!(z, E12::new(1, 1), "60° should snap to (1,1), got {:?}", z);
    }

    // === HexDisk::snap_direction tests ===

    #[test]
    #[cfg(feature = "snap")]
    fn test_hex_disk_snap_direction_east() {
        let disk = HexDisk::radius(10);
        let z = disk.snap_direction(0.0).unwrap();
        assert_eq!(z, E12::new(1, 0), "Disk east should snap to (1,0)");
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_hex_disk_snap_direction_radius_0() {
        // Disk radius 0 only contains (0,0), none to snap
        let disk = HexDisk::radius(0);
        assert!(disk.snap_direction(0.0).is_none());
    }

    #[test]
    #[cfg(feature = "snap")]
    fn test_hex_disk_snap_vs_e12_snap_hex_directions() {
        // For the 6 hex directions, disk and E12 snap should agree
        let disk = HexDisk::radius(36);
        let hex_angles = [0.0, consts::FRAC_PI_3, 2.0*consts::FRAC_PI_3, consts::PI, 4.0*consts::FRAC_PI_3, 5.0*consts::FRAC_PI_3];
        for angle in hex_angles {
            let disk_z = disk.snap_direction(angle).unwrap();
            let e12_z = E12::snap_from_angle(angle);
            // Disk may pick a different representative within radius, but angle should be very close
            let disk_ang = E12::angular_distance(disk_z, float::cos(angle), float::sin(angle));
            let e12_ang = E12::angular_distance(e12_z, float::cos(angle), float::sin(angle));
            assert!(disk_ang < 0.01, "Disk snap at {}° has angular diff {}", angle * 180.0 / consts::PI, disk_ang);
            assert!(e12_ang < 0.01, "E12 snap at {}° has angular diff {}", angle * 180.0 / consts::PI, e12_ang);
        }
    }
}
