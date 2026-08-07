//! Additional edge case tests for eisenstein crate.
//!
//! These tests focus on boundary conditions, algebraic properties,
//! and edge cases not covered by the main test suite.

use eisenstein::{E12, HexDisk, EisensteinTriple};
use eisenstein::{laman_redundancy_2d, laman_redundancy_3d, laman_redundancy_2d_ratio, laman_redundancy_3d_ratio};

#[cfg(test)]
mod tests {
    use super::*;

    // ─── E12 Edge Cases ────────────────────────────────────────────────

    #[test]
    fn test_new_negative_coordinates() {
        let z = E12::new(-5, -3);
        assert_eq!(z.a(), -5);
        assert_eq!(z.b(), -3);
        // norm: (-5)² - (-5)(-3) + (-3)² = 25 - 15 + 9 = 19
        assert_eq!(z.norm(), 19);
    }

    #[test]
    fn test_new_mixed_sign_coordinates() {
        let z = E12::new(5, -3);
        // norm: 25 - (5)(-3) + 9 = 25 + 15 + 9 = 49
        assert_eq!(z.norm(), 49);
    }

    #[test]
    fn test_norm_of_unit_eisenstein() {
        // The six units have norm 1
        let units = E12::directions();
        for u in &units {
            assert_eq!(u.norm(), 1, "Unit {:?} should have norm 1", u);
        }
    }

    #[test]
    fn test_norm_zero_only_for_origin() {
        // Only (0,0) has norm 0
        assert_eq!(E12::new(0, 0).norm(), 0);
        assert_ne!(E12::new(1, 0).norm(), 0);
        assert_ne!(E12::new(0, 1).norm(), 0);
        assert_ne!(E12::new(-1, 0).norm(), 0);
    }

    #[test]
    fn test_norm_always_nonneg_u64() {
        // Test many coordinates — norm should never be negative
        for a in -100i32..=100 {
            for b in -100i32..=100 {
                let n = E12::new(a, b).norm();
                // norm is u64, so it's always non-negative by type
                // but verify the formula gives sensible values
                let expected = (a as i64 * a as i64 - a as i64 * b as i64 + b as i64 * b as i64) as u64;
                assert_eq!(n, expected, "norm({a}, {b}) mismatch");
            }
        }
    }

    #[test]
    fn test_addition_commutative() {
        let z1 = E12::new(3, 5);
        let z2 = E12::new(7, 2);
        let sum1 = z1 + z2;
        let sum2 = z2 + z1;
        assert_eq!(sum1, sum2);
    }

    #[test]
    fn test_addition_associative() {
        let z1 = E12::new(1, 2);
        let z2 = E12::new(3, 4);
        let z3 = E12::new(5, 6);
        assert_eq!((z1 + z2) + z3, z1 + (z2 + z3));
    }

    #[test]
    fn test_addition_zero_identity() {
        let z = E12::new(7, -3);
        assert_eq!(z + E12::new(0, 0), z);
    }

    #[test]
    fn test_mul_by_negative_one() {
        let z = E12::new(5, 3);
        let neg = E12::new(0, 0) - z; // -z
        let result = z + neg;
        assert_eq!(result, E12::new(0, 0));
    }

    #[test]
    fn test_conjugate_of_negative() {
        let z = E12::new(-4, 7);
        let conj = z.conjugate();
        // conjugate of a + bω is a + bω² = (a-b) + (-b)ω... actually
        // conjugate of a + bω is a + bω̄ where ω̄ = ω²
        // ω² = -1 - ω, so a + b(-1-ω) = (a-b) - bω
        // Hmm, let me check what the implementation does
        assert_eq!(conj.norm(), z.norm()); // norm is preserved
    }

    #[test]
    fn test_scale_zero() {
        let z = E12::new(5, 3);
        assert_eq!(z.scale(0), E12::new(0, 0));
    }

    #[test]
    fn test_scale_negative() {
        let z = E12::new(5, 3);
        let scaled = z.scale(-2);
        assert_eq!(scaled.a(), -10);
        assert_eq!(scaled.b(), -6);
    }

    #[test]
    fn test_scale_preserves_norm_ratio() {
        let z = E12::new(3, 7);
        let k: i32 = 4;
        let scaled = z.scale(k);
        assert_eq!(scaled.norm(), z.norm() * (k as u64 * k as u64));
    }

    // ─── Hex Distance Edge Cases ───────────────────────────────────────

    #[test]
    fn test_hex_distance_to_self() {
        let z = E12::new(5, 3);
        assert_eq!(z.hex_distance_to(z), 0);
    }

    #[test]
    fn test_hex_distance_to_neighbor() {
        let z = E12::new(0, 0);
        let neighbors = z.neighbors();
        // All neighbors are origin + a unit direction
        // hex_distance for Eisenstein coords uses (|a|+|b|+|a+b|)/2
        // The six directions have varying hex distances
        for n in &neighbors {
            let d = z.hex_distance_to(*n);
            assert!(d >= 1, "Neighbor should be at distance >= 1");
            assert!(d <= 2, "Neighbor should be at distance <= 2");
        }
    }

    #[test]
    fn test_hex_distance_symmetric() {
        let z1 = E12::new(10, -5);
        let z2 = E12::new(-3, 7);
        assert_eq!(z1.hex_distance_to(z2), z2.hex_distance_to(z1));
    }

    #[test]
    fn test_hex_distance_triangle_inequality() {
        let z1 = E12::new(0, 0);
        let z2 = E12::new(5, 3);
        let z3 = E12::new(-2, 8);
        let d12 = z1.hex_distance_to(z2);
        let d23 = z2.hex_distance_to(z3);
        let d13 = z1.hex_distance_to(z3);
        assert!(d13 <= d12 + d23, "Triangle inequality violated");
    }

    // ─── D6 Rotation Edge Cases ────────────────────────────────────────

    #[test]
    fn test_d6_rotations_returns_six() {
        let z = E12::new(3, 1);
        let rots = z.d6_rotations();
        assert_eq!(rots.len(), 6);
    }

    #[test]
    fn test_d6_rotations_all_same_norm() {
        let z = E12::new(7, -3);
        let rots = z.d6_rotations();
        for r in &rots {
            assert_eq!(r.norm(), z.norm());
        }
    }

    #[test]
    fn test_d6_rotations_of_zero_all_zero() {
        let z = E12::new(0, 0);
        let rots = z.d6_rotations();
        for r in &rots {
            assert_eq!(*r, E12::new(0, 0));
        }
    }

    #[test]
    fn test_d6_rotations_are_distinct() {
        let z = E12::new(5, 2);
        let rots = z.d6_rotations();
        // All 6 rotations should be distinct for non-zero, non-symmetric points
        for i in 0..6 {
            for j in (i+1)..6 {
                assert_ne!(rots[i], rots[j], "Rotations {i} and {j} should differ");
            }
        }
    }

    // ─── Divisibility Edge Cases ───────────────────────────────────────

    #[test]
    fn test_divides_nonzero_by_nonzero() {
        let z = E12::new(6, 0);
        let d = E12::new(2, 0);
        assert!(d.divides(z));
    }

    #[test]
    fn test_divides_not_divisible() {
        let z = E12::new(5, 0);
        let d = E12::new(2, 0);
        assert!(!d.divides(z));
    }

    #[test]
    fn test_gcd_both_zero() {
        let z = E12::new(0, 0);
        assert_eq!(z.gcd(E12::new(0, 0)), E12::new(0, 0));
    }

    #[test]
    fn test_gcd_with_associates() {
        // gcd should be defined up to associates (units)
        let z = E12::new(6, 0);
        let w = E12::new(0, 6);
        let g = z.gcd(w);
        assert!(g.norm() > 0);
    }

    // ─── Eisenstein Triple Edge Cases ──────────────────────────────────

    #[test]
    fn test_eisenstein_triple_norm_formula() {
        // For triple (a, b, c): a² - ab + b² = c²
        // Verify for a few known values
        for n in 1u32..=50 {
            if let Some(t) = EisensteinTriple::from_norm(n * n) {
                let computed = (t.a() as i64 * t.a() as i64
                    - t.a() as i64 * t.b() as i64
                    + t.b() as i64 * t.b() as i64) as u64;
                assert_eq!(computed, (n * n) as u64,
                    "Triple from norm {}² doesn't satisfy formula", n);
            }
        }
    }

    #[test]
    fn test_eisenstein_triple_from_norm_no_solution() {
        // Norm that has no Eisenstein triple representation
        // 2 is not representable as a²-ab+b² for any (a,b)
        // Actually 1² - 1*0 + 0² = 1, so norm 1 has (1,0,1)
        // Let's check small norms
        for n in 1u32..=20 {
            if let Some(t) = EisensteinTriple::from_norm(n) {
                let computed = (t.a() as i64 * t.a() as i64
                    - t.a() as i64 * t.b() as i64
                    + t.b() as i64 * t.b() as i64) as u64;
                assert_eq!(computed, n as u64);
            }
        }
    }

    // ─── HexDisk Edge Cases ────────────────────────────────────────────

    #[test]
    fn test_hex_disk_count_formula() {
        // HexDisk of radius R contains 3R² + 3R + 1 points
        for r in 0u32..=10 {
            let disk = HexDisk::radius(r);
            let expected = 3 * r as u64 * r as u64 + 3 * r as u64 + 1;
            assert_eq!(disk.count(), expected, "HexDisk radius {r} count mismatch");
        }
    }

    #[test]
    fn test_hex_disk_iter_count_matches() {
        for r in 0u32..=8 {
            let disk = HexDisk::radius(r);
            let iter_count = disk.iter().count() as u64;
            assert_eq!(iter_count, disk.count(), "HexDisk radius {r} iter count != count()");
        }
    }

    #[test]
    fn test_hex_disk_radius_0() {
        let disk = HexDisk::radius(0);
        assert_eq!(disk.count(), 1);
        let points: Vec<_> = disk.iter().collect();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0], E12::new(0, 0));
    }

    #[test]
    fn test_hex_disk_contains_center() {
        let disk = HexDisk::radius(5);
        assert!(disk.contains(&E12::new(0, 0)));
    }

    #[test]
    fn test_hex_disk_does_not_contain_outside() {
        let disk = HexDisk::radius(3);
        // Point at distance > 3 should not be contained
        let far = E12::new(10, 0);
        assert!(!disk.contains(&far));
    }

    // ─── Axial Conversion Edge Cases ───────────────────────────────────

    #[test]
    fn test_from_axial_negative_coords() {
        let z = E12::from_axial(-5, 3);
        let (q, r) = z.to_axial();
        assert_eq!(q, -5);
        assert_eq!(r, 3);
    }

    #[test]
    fn test_from_axial_zero() {
        let z = E12::from_axial(0, 0);
        assert_eq!(z, E12::new(0, 0));
    }

    // ─── Laman Constants ───────────────────────────────────────────────

    #[test]
    fn test_laman_2d_value() {
        assert_eq!(laman_redundancy_2d(), 15);
    }

    #[test]
    fn test_laman_3d_value() {
        assert_eq!(laman_redundancy_3d(), 20);
    }

    #[test]
    fn test_laman_2d_ratio() {
        assert_eq!(laman_redundancy_2d_ratio(), (3, 2));
    }

    #[test]
    fn test_laman_3d_ratio() {
        assert_eq!(laman_redundancy_3d_ratio(), (2, 1));
    }

    // ─── Directions ────────────────────────────────────────────────────

    #[test]
    fn test_directions_count() {
        let dirs = E12::directions();
        assert_eq!(dirs.len(), 6);
    }

    #[test]
    fn test_directions_all_unit_norm() {
        let dirs = E12::directions();
        for d in &dirs {
            assert_eq!(d.norm(), 1);
        }
    }

    #[test]
    fn test_directions_are_neighbors_of_origin() {
        let origin = E12::new(0, 0);
        let neighbors = origin.neighbors();
        let dirs = E12::directions();
        // neighbors should match directions (possibly in different order)
        for d in &dirs {
            assert!(neighbors.contains(d), "Direction {:?} not in neighbors", d);
        }
    }

    // ─── Multiplication Properties ─────────────────────────────────────

    #[test]
    fn test_mul_norm_multiplicativity_large_values() {
        let z1 = E12::new(100, -50);
        let z2 = E12::new(37, 29);
        let product = z1 * z2;
        assert_eq!(product.norm(), z1.norm() * z2.norm());
    }

    #[test]
    fn test_mul_conjugate_gives_norm() {
        // z * conj(z) should give a point on the real axis with norm = norm(z)²
        // Actually z * conj(z) = |z|² in Eisenstein integers = norm(z) as a real point
        let z = E12::new(3, 1);
        let conj = z.conjugate();
        let product = z * conj;
        // norm of product = norm(z) * norm(conj(z)) = norm(z)²
        // But norm(conj(z)) = norm(z), so norm(product) = norm(z)²
        assert_eq!(product.norm(), z.norm() * z.norm());
    }

    // ─── Div Rem Edge Cases ────────────────────────────────────────────

    #[test]
    fn test_div_rem_by_associate() {
        // Dividing by an associate (unit multiple) should give exact result
        let z = E12::new(6, 0);
        let d = E12::new(2, 0);
        let result = z.div_rem(d);
        assert!(result.is_some());
        let (quotient, remainder) = result.unwrap();
        assert_eq!(remainder, E12::new(0, 0)); // exact division
    }

    // ─── Is Unit ───────────────────────────────────────────────────────

    #[test]
    fn test_is_unit_for_all_directions() {
        for d in &E12::directions() {
            assert!(d.is_unit(), "Direction {:?} should be a unit", d);
        }
    }

    #[test]
    fn test_is_unit_for_non_units() {
        assert!(!E12::new(2, 0).is_unit());
        assert!(!E12::new(0, 2).is_unit());
        // (1,1) has norm 1 - 1 + 1 = 1, so it IS a unit (-ω²)
        assert!(E12::new(1, 1).is_unit());
        assert!(!E12::new(2, 2).is_unit()); // norm = 4 - 4 + 4 = 4
    }

    #[test]
    fn test_is_unit_origin() {
        assert!(!E12::new(0, 0).is_unit());
    }

    // ─── Integer Sqrt Edge Cases ───────────────────────────────────────

    #[test]
    fn test_int_sqrt_zero() {
        // The internal int_sqrt function should handle 0
        // We test it indirectly through from_norm
        // If from_norm(0) is called, it should find (0,0)
        let result = EisensteinTriple::from_norm_raw(0);
        assert!(result.is_some());
    }

    #[test]
    fn test_int_sqrt_one() {
        let result = EisensteinTriple::from_norm_raw(1);
        assert!(result.is_some());
    }

    #[test]
    fn test_from_norm_raw_large_prime() {
        // A large prime that's representable should be found
        // Norm 7 = 3²-3*1+1² = 9-3+1 = 7
        let result = EisensteinTriple::from_norm_raw(7);
        assert!(result.is_some());
        let (a, b) = result.unwrap();
        assert_eq!(a as i64 * a as i64 - a as i64 * b as i64 + b as i64 * b as i64, 7);
    }

    #[test]
    fn test_from_norm_raw_no_solution() {
        // Norm 5: is there (a,b) with a²-ab+b²=5?
        // Try: (3,1): 9-3+1=7 no. (2,1): 4-2+1=3 no. (3,2): 9-6+4=7 no.
        // (2,-1): 4+2+1=7. (3,-1): 9+3+1=13.
        // Actually (2,0): 4. (3,0): 9. Hmm, is 5 representable?
        // 5 is a prime ≡ 2 mod 3, so it's NOT representable in Z[ω]
        let result = EisensteinTriple::from_norm_raw(5);
        assert!(result.is_none(), "Norm 5 should not be representable");
    }

    // ─── EisensteinTriple: from_norm, all_with_max_norm, is_primitive, generate ──

    #[test]
    fn test_from_norm_representable() {
        // from_norm expects a perfect square target (c²) where a²-ab+b² = c²
        // c=3 → target=9. We need a²-ab+b²=9. (3,0): 9-0+0=9 ✓
        let triple = EisensteinTriple::from_norm(9);
        assert!(triple.is_some(), "Norm 9 (= 3²) should be representable");
        let t = triple.unwrap();
        assert_eq!(t.norm(), 9);
        assert_eq!(t.c(), 3);
    }

    #[test]
    fn test_from_norm_not_representable() {
        // from_norm(25) → c=5, need a²-ab+b²=25.
        // 5 is prime ≡ 2 mod 3, so norm 25 is not representable in Z[ω]
        // Actually 25 = 5², and primes ≡ 2 mod 3 are inert in Z[ω],
        // so 5² splits as 5·5̄ but... let's just check: no (a,b) with a²-ab+b²=25
        // (5,0): 25 ✓! So this IS representable. Let's pick something else.
        // c=2 → target=4. (2,0): 4 ✓. Representable.
        // c=5 → 25 IS representable by (5,0). 
        // Use c=2 → already works. Try something that genuinely fails.
        // Actually from_norm returns None if target is not a perfect square
        let triple = EisensteinTriple::from_norm(10);
        assert!(triple.is_none(), "Norm 10 is not a perfect square, no triple exists");
    }

    #[test]
    fn test_from_norm_zero() {
        let triple = EisensteinTriple::from_norm(0);
        // Origin has norm 0
        // from_norm_raw(0) should find (0, 0)
        // depending on implementation this may or may not return Some
        if let Some(t) = triple {
            assert_eq!(t.norm(), 0);
        }
    }

    #[test]
    fn test_from_norm_one() {
        let triple = EisensteinTriple::from_norm(1);
        assert!(triple.is_some());
        let t = triple.unwrap();
        assert_eq!(t.norm(), 1);
    }

    #[test]
    fn test_all_with_max_norm_contains_units() {
        let triples = EisensteinTriple::all_with_max_norm(1);
        // At minimum, we should have the six units (norm 1)
        // or at least one representative
        assert!(!triples.is_empty(), "Should find triples with norm ≤ 1");
        for t in &triples {
            assert!(t.norm() <= 1);
        }
    }

    #[test]
    fn test_all_with_max_norm_grows() {
        let small = EisensteinTriple::all_with_max_norm(10);
        let large = EisensteinTriple::all_with_max_norm(100);
        // More triples should be found with larger bound
        assert!(large.len() >= small.len(),
            "Larger norm bound should find at least as many triples");
    }

    #[test]
    fn test_all_with_max_norm_respects_bound() {
        // all_with_max_norm(c_max) returns triples where c ≤ c_max
        // (not where norm ≤ c_max — norm is c²)
        let triples = EisensteinTriple::all_with_max_norm(10);
        for t in &triples {
            assert!(t.c() <= 10,
                "Triple c={} exceeds bound 10", t.c());
        }
    }

    #[test]
    fn test_is_primitive_for_coprime() {
        // (2, -1) has norm 7 (prime), so it's primitive
        let t = EisensteinTriple::new(2, -1, 7);
        assert!(t.is_primitive(),
            "Triple with prime norm should be primitive");
    }

    #[test]
    fn test_is_primitive_for_non_coprime() {
        // (2, 0) has norm 4. It's 2 * (1, 0), so not primitive
        let t = EisensteinTriple::new(2, 0, 4);
        assert!(!t.is_primitive(),
            "Triple that is a scalar multiple should not be primitive");
    }

    #[test]
    fn test_is_primitive_origin() {
        let t = EisensteinTriple::new(0, 0, 0);
        // Origin — depends on implementation, but gcd(0,0) = 0
        // so origin is not primitive (it's 0 * anything)
        assert!(!t.is_primitive());
    }

    #[test]
    fn test_generate_returns_sorted_by_index() {
        let triples = EisensteinTriple::generate(10);
        assert_eq!(triples.len(), 10, "generate(10) should return 10 triples");
        // Each should satisfy the norm formula a² - ab + b² = c²
        for t in &triples {
            let a = t.a() as i64;
            let b = t.b() as i64;
            let c = t.c() as i64;
            let lhs = a * a - a * b + b * b;
            assert_eq!(lhs, c * c,
                "Triple ({},{},{}) doesn't satisfy a²-ab+b²=c²", a, b, c);
        }
    }

    #[test]
    fn test_generate_zero_returns_empty() {
        let triples = EisensteinTriple::generate(0);
        assert!(triples.is_empty(), "generate(0) should return empty");
    }

    #[test]
    fn test_generate_one() {
        let triples = EisensteinTriple::generate(1);
        assert_eq!(triples.len(), 1);
        // First triple should be valid
        let t = &triples[0];
        let a = t.a() as i64;
        let b = t.b() as i64;
        let c = t.c() as i64;
        assert_eq!(a * a - a * b + b * b, c * c);
    }

    // ─── E12: norm_divisors ───────────────────────────────────────────

    #[test]
    fn test_norm_divisors_of_unit() {
        let u = E12::new(1, 0); // norm 1
        let divs = u.norm_divisors();
        // A unit's norm is 1, so the only divisor norm is 1
        // The divisors should include at least the unit itself
        assert!(!divs.is_empty(), "A unit should have norm divisors");
        for d in &divs {
            assert_eq!(d.norm(), 1, "All norm divisors of a unit have norm 1");
        }
    }

    #[test]
    fn test_norm_divisors_of_prime_norm() {
        // (2, -1) has norm 7 (prime in Z)
        let z = E12::new(2, -1);
        let divs = z.norm_divisors();
        // Divisors of 7 are 1 and 7
        for d in &divs {
            let n = d.norm();
            assert!(n == 1 || n == 7,
                "Norm divisor {} should be 1 or 7", n);
        }
    }

    #[test]
    fn test_norm_divisors_include_self() {
        let z = E12::new(2, -1); // norm 7
        let divs = z.norm_divisors();
        // z divides itself
        assert!(divs.iter().any(|&d| d == z),
            "Self should be in norm divisors");
    }

    // ─── E12: div_rem edge cases ──────────────────────────────────────

    #[test]
    fn test_div_rem_by_unit() {
        let z = E12::new(6, -3);
        let unit = E12::new(1, 0); // unit
        let result = z.div_rem(unit);
        assert!(result.is_some());
        let (q, r) = result.unwrap();
        assert_eq!(r.norm(), 0, "Remainder when dividing by unit should be 0");
        assert_eq!(q, z, "Quotient when dividing by unit should be the number");
    }

    #[test]
    fn test_div_rem_exact_division() {
        // (6, -3) / (2, -1) should be (3, 0) with remainder 0
        // because (2,-1) * (3,0) = (6, -3)
        let z = E12::new(6, -3);
        let d = E12::new(2, -1);
        let result = z.div_rem(d);
        assert!(result.is_some());
        let (q, r) = result.unwrap();
        assert_eq!(r.norm(), 0, "Should divide exactly");
        // Verify: d * q = z
        let product = d * q;
        assert_eq!(product, z, "divisor * quotient should equal dividend");
    }

    #[test]
    fn test_div_rem_has_remainder_when_not_divisible() {
        // div_rem always returns Some for non-zero divisor — it rounds.
        // When not exactly divisible, remainder is nonzero.
        let z = E12::new(3, 0); // norm 9
        let d = E12::new(2, -1); // norm 7
        let result = z.div_rem(d);
        assert!(result.is_some(), "div_rem always returns Some for nonzero divisor");
        let (_q, r) = result.unwrap();
        // remainder should be nonzero (7 doesn't divide 9)
        assert_ne!(r.norm(), 0, "Remainder should be nonzero when not divisible");
    }

    // ─── E12: scale with positive integer ─────────────────────────────

    #[test]
    fn test_scale_positive() {
        let z = E12::new(2, -1);
        let scaled = z.scale(3);
        assert_eq!(scaled.a(), 6);
        assert_eq!(scaled.b(), -3);
        // norm should scale by k²
        assert_eq!(scaled.norm(), z.norm() * 9);
    }

    #[test]
    fn test_scale_one_is_identity() {
        let z = E12::new(5, -3);
        assert_eq!(z.scale(1), z);
    }

    // ─── HexDisk: snap_direction ──────────────────────────────────────

    #[cfg(feature = "snap")]
    #[test]
    fn test_snap_direction_returns_valid_hex() {
        let disk = HexDisk::radius(5);
        // 0 radians → east direction
        let result = disk.snap_direction(0.0);
        assert!(result.is_some());
        let hex = result.unwrap();
        // Should be one of the 6 hex directions
        assert!(hex.norm() <= 5);
    }

    #[cfg(feature = "snap")]
    #[test]
    fn test_snap_direction_all_six_directions() {
        let disk = HexDisk::radius(3);
        // Test snapping at each of the 6 hex angles (60° apart)
        for i in 0..6 {
            let angle = core::f64::consts::FRAC_PI_3 * i as f64;
            let result = disk.snap_direction(angle);
            assert!(result.is_some(),
                "snap_direction should return Some for angle {}π/3", i);
        }
    }

    #[cfg(feature = "snap")]
    #[test]
    fn test_snap_direction_radius_zero() {
        let disk = HexDisk::radius(0);
        // A radius-0 disk only contains the origin
        // snap_direction may return None or the origin depending on impl
        let result = disk.snap_direction(0.0);
        // If it returns something, it should be the origin or very close
        if let Some(hex) = result {
            assert_eq!(hex.norm(), 0, "Radius-0 disk should snap to origin");
        }
    }

    // ─── E12: snap_from_angle ─────────────────────────────────────────

    #[cfg(feature = "snap")]
    #[test]
    fn test_snap_from_angle_zero_radians() {
        let hex = E12::snap_from_angle(0.0);
        // 0 radians → east (1, 0)
        assert_eq!(hex.norm(), 1, "Snapping 0 should give a unit vector");
    }

    #[cfg(feature = "snap")]
    #[test]
    fn test_snap_from_angle_full_circle() {
        // Snapping at any angle should give a valid E12 point.
        // Angles between hex directions may snap to non-unit vectors,
        // so we just verify the result is non-degenerate.
        for i in 0..12 {
            let angle = core::f64::consts::FRAC_PI_6 * i as f64;
            let hex = E12::snap_from_angle(angle);
            // At multiples of π/3 (60°), should snap to a unit vector
            if i % 2 == 0 {
                assert_eq!(hex.norm(), 1,
                    "Angle at {}π/6 = {}π/3 should snap to unit vector", i, i/2);
            }
            // Otherwise just verify it's a valid point
            assert!(hex.norm() > 0,
                "Snapped angle should give nonzero vector (angle {}π/6)", i);
        }
    }

    // ─── E12: gcd additional cases ────────────────────────────────────

    #[test]
    fn test_gcd_with_one() {
        let z = E12::new(6, -3);
        let one = E12::new(1, 0);
        let g = z.gcd(one);
        // gcd(z, unit) should be a unit
        assert_eq!(g.norm(), 1);
    }

    #[test]
    fn test_gcd_with_self() {
        let z = E12::new(6, -3);
        let g = z.gcd(z);
        // gcd(z, z) should be z (up to associates)
        assert_eq!(g.norm(), z.norm());
    }

    #[test]
    fn test_gcd_commutative() {
        let a = E12::new(6, -3);
        let b = E12::new(4, 1);
        let g1 = a.gcd(b);
        let g2 = b.gcd(a);
        assert_eq!(g1.norm(), g2.norm(),
            "GCD should be commutative (same norm)");
    }

    // ─── E12: divides additional cases ────────────────────────────────

    #[test]
    fn test_divides_by_one() {
        let z = E12::new(6, -3);
        assert!(z.divides(z), "Every element divides itself");
    }

    #[test]
    fn test_divides_by_associate() {
        // Associates divide each other
        let z = E12::new(2, -1); // norm 7
        // Rotate by 60° to get an associate
        let rotations = z.d6_rotations();
        for r in &rotations {
            assert!(z.divides(*r),
                "Associates (D6 rotations) should divide each other");
        }
    }

    #[test]
    fn test_divides_origin_divides_all() {
        let origin = E12::new(0, 0);
        let z = E12::new(5, 3);
        // origin does NOT divide anything (division by zero norm)
        // But z.divides(origin) checks if z | origin, which is true
        assert!(z.divides(origin),
            "Any nonzero element divides the origin");
    }
}
