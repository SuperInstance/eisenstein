#![no_std]

//! Zero-drift hexagonal lattice constraints via Eisenstein integers.
//!
//! Exact integer arithmetic for safety-critical systems. No floats, no unsafe,
//! no dependencies. Just pure hexagonal math.

use core::fmt;

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
}
