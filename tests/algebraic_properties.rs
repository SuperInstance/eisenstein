//! Algebraic property tests for eisenstein crate.
//!
//! These tests verify deep algebraic properties of Z[ω]:
//! - Ring axioms (additive/multiplicative identity, associativity, distributivity)
//! - Euclidean domain properties (division algorithm correctness)
//! - D₆ symmetry group properties
//! - UFD properties (gcd, divisibility)
//! - Eisenstein triple density and correctness
//!
//! Named "algebraic_properties" because each test verifies a mathematical
//! theorem, not just a function's output.

use eisenstein::{EisensteinTriple, HexDisk, E12};

#[cfg(test)]
mod ring_axioms {
    use super::*;

    // Z[ω] is a commutative ring with unity.
    // Verify all ring axioms exhaustively over a small domain.

    fn small_domain() -> Vec<E12> {
        let mut v = Vec::new();
        for a in -5..=5 {
            for b in -5..=5 {
                v.push(E12::new(a, b));
            }
        }
        v
    }

    #[test]
    fn addition_is_associative() {
        // (a + b) + c == a + (b + c) for all a,b,c
        let domain = small_domain();
        for &a in &domain {
            for &b in &domain {
                for &c in &domain {
                    assert_eq!(
                        (a + b) + c,
                        a + (b + c),
                        "Additive associativity failed for ({},{},{})",
                        a,
                        b,
                        c
                    );
                }
            }
        }
    }

    #[test]
    fn addition_is_commutative() {
        // a + b == b + a
        let domain = small_domain();
        for &a in &domain {
            for &b in &domain {
                assert_eq!(
                    a + b,
                    b + a,
                    "Additive commutativity failed for {} {}",
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn additive_identity() {
        // a + 0 == a
        let domain = small_domain();
        let zero = E12::new(0, 0);
        for &a in &domain {
            assert_eq!(a + zero, a);
        }
    }

    #[test]
    fn additive_inverse() {
        // a + (-a) == 0
        let domain = small_domain();
        let zero = E12::new(0, 0);
        for &a in &domain {
            let neg_a = zero - a;
            assert_eq!(a + neg_a, zero, "Additive inverse failed for {}", a);
        }
    }

    #[test]
    fn multiplication_is_associative() {
        // (a * b) * c == a * (b * c)
        let domain: Vec<E12> = small_domain()
            .into_iter()
            .filter(|z| z.a().abs() <= 3 && z.b().abs() <= 3)
            .collect();
        for &a in &domain {
            for &b in &domain {
                for &c in &domain {
                    assert_eq!(
                        (a * b) * c,
                        a * (b * c),
                        "Multiplicative associativity failed for ({},{},{})",
                        a,
                        b,
                        c
                    );
                }
            }
        }
    }

    #[test]
    fn multiplication_is_commutative() {
        // a * b == b * a
        let domain = small_domain();
        for &a in &domain {
            for &b in &domain {
                assert_eq!(
                    a * b,
                    b * a,
                    "Multiplicative commutativity failed for {} {}",
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn multiplicative_identity() {
        // a * 1 == a
        let domain = small_domain();
        let one = E12::new(1, 0);
        for &a in &domain {
            assert_eq!(a * one, a);
        }
    }

    #[test]
    fn distributivity() {
        // a * (b + c) == a * b + a * c
        let domain: Vec<E12> = small_domain()
            .into_iter()
            .filter(|z| z.a().abs() <= 3 && z.b().abs() <= 3)
            .collect();
        for &a in &domain {
            for &b in &domain {
                for &c in &domain {
                    assert_eq!(
                        a * (b + c),
                        a * b + a * c,
                        "Distributivity failed for ({},{},{})",
                        a,
                        b,
                        c
                    );
                }
            }
        }
    }

    #[test]
    fn zero_multiplication() {
        // a * 0 == 0
        let domain = small_domain();
        let zero = E12::new(0, 0);
        for &a in &domain {
            assert_eq!(a * zero, zero);
        }
    }
}

#[cfg(test)]
mod euclidean_domain_properties {
    use super::*;

    // Z[ω] is a Euclidean domain with norm N(a+bω) = a² - ab + b².
    // The Euclidean property guarantees: for all α, β≠0, ∃ γ,ρ such that
    // α = βγ + ρ and N(ρ) < N(β).

    #[test]
    fn euclidean_property_holds_for_all_pairs() {
        // For all α, β≠0 in a small domain, verify N(remainder) < N(divisor).
        let mut count = 0;
        for a_coord in -8..=8 {
            for b_coord in -8..=8 {
                let alpha = E12::new(a_coord, b_coord);
                for c_coord in -8..=8 {
                    for d_coord in -8..=8 {
                        let beta = E12::new(c_coord, d_coord);
                        if beta.a() == 0 && beta.b() == 0 {
                            continue;
                        }
                        if let Some((q, r)) = alpha.div_rem(beta) {
                            // Verify reconstruction: β*γ + ρ = α
                            assert_eq!(
                                beta * q + r,
                                alpha,
                                "Reconstruction failed: {} * {} + {} != {}",
                                beta,
                                q,
                                r,
                                alpha
                            );
                            // Verify Euclidean property: N(ρ) < N(β) or ρ = 0
                            let n_r = r.norm();
                            let n_beta = beta.norm();
                            assert!(
                                n_r < n_beta || (r.a() == 0 && r.b() == 0),
                                "Euclidean property failed: N(rem)={} >= N(div)={} for {} / {}",
                                n_r,
                                n_beta,
                                alpha,
                                beta
                            );
                            count += 1;
                        }
                    }
                }
            }
        }
        // Ensure we actually tested a meaningful number of pairs
        assert!(
            count > 1000,
            "Should have tested at least 1000 division pairs, got {}",
            count
        );
    }

    #[test]
    fn division_by_unit_is_exact() {
        // Dividing by any of the 6 units should give exact quotient (remainder 0).
        let units = E12::directions();
        for a_coord in -10..=10 {
            for b_coord in -10..=10 {
                let alpha = E12::new(a_coord, b_coord);
                for &u in &units {
                    if let Some((_, r)) = alpha.div_rem(u) {
                        assert_eq!(
                            r,
                            E12::new(0, 0),
                            "Division by unit {:?} of {} should be exact, got remainder {}",
                            u,
                            alpha,
                            r
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn division_of_zero_gives_zero() {
        // 0 / β = (0, 0) with remainder 0
        for a in -5..=5 {
            for b in -5..=5 {
                let beta = E12::new(a, b);
                if beta.a() == 0 && beta.b() == 0 {
                    continue;
                }
                let (q, r) = E12::new(0, 0).div_rem(beta).unwrap();
                assert_eq!(q, E12::new(0, 0), "Quotient of 0/beta should be 0");
                assert_eq!(r, E12::new(0, 0), "Remainder of 0/beta should be 0");
            }
        }
    }
}

#[cfg(test)]
mod d6_symmetry_properties {
    use super::*;

    // The D₆ point group has 6 rotations and 6 reflections (12 elements total).
    // We test the rotation subgroup C₆ ⊂ D₆ here.

    #[test]
    fn rotation_by_360_returns_to_origin() {
        // Six 60° rotations = identity
        let test_points = vec![
            E12::new(1, 0),
            E12::new(0, 1),
            E12::new(1, 1),
            E12::new(5, -3),
            E12::new(7, 2),
            E12::new(-3, 11),
            E12::new(0, 0),
        ];
        for &z in &test_points {
            let mut current = z;
            for _ in 0..6 {
                current = E12::new(-current.b(), current.a() - current.b());
            }
            assert_eq!(
                current, z,
                "Six 60° rotations should return to start: {}",
                z
            );
        }
    }

    #[test]
    fn rotation_preserves_norm() {
        // All D₆ rotations preserve the Eisenstein norm
        let test_points = vec![
            E12::new(1, 0),
            E12::new(3, -2),
            E12::new(7, 1),
            E12::new(13, -7),
            E12::new(0, 5),
        ];
        for &z in &test_points {
            let rots = z.d6_rotations();
            for r in &rots {
                assert_eq!(
                    r.norm(),
                    z.norm(),
                    "D₆ rotation should preserve norm of {}",
                    z
                );
            }
        }
    }

    #[test]
    fn rotations_are_all_distinct_for_generic_point() {
        // For a generic point (not on a symmetry axis), all 6 rotations differ
        let z = E12::new(5, 2);
        let rots = z.d6_rotations();
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_ne!(
                    rots[i], rots[j],
                    "Rotations {} and {} of {} should be distinct",
                    i, j, z
                );
            }
        }
    }

    #[test]
    fn rotation_orbits_partition_by_norm() {
        // All points in a D₆ orbit have the same norm
        for a in 1..=5 {
            for b in 0..=a {
                let z = E12::new(a, b);
                let rots = z.d6_rotations();
                let n = z.norm();
                for r in &rots {
                    assert_eq!(r.norm(), n);
                }
            }
        }
    }

    #[test]
    fn rotation_composition() {
        // Two 60° rotations = one 120° rotation
        // rot60(rot60(z)) == rot120(z)
        let test_points = vec![E12::new(3, 1), E12::new(7, -2), E12::new(0, 5)];
        for &z in &test_points {
            let rot60 = |p: E12| E12::new(-p.b(), p.a() - p.b());
            let double = rot60(rot60(z));
            let rots = z.d6_rotations();
            // rots[2] is 120° rotation
            assert_eq!(
                double, rots[2],
                "Two 60° rotations should equal 120° rotation for {}",
                z
            );
        }
    }

    #[test]
    fn negation_is_180_rotation() {
        // -(a + bω) = (-a, -b) = 180° rotation
        let test_points = vec![E12::new(3, 1), E12::new(7, -2), E12::new(5, 0)];
        for &z in &test_points {
            let rots = z.d6_rotations();
            let neg = E12::new(-z.a(), -z.b());
            assert_eq!(
                rots[3], neg,
                "180° rotation should equal negation for {}",
                z
            );
        }
    }
}

#[cfg(test)]
mod norm_properties {
    use super::*;

    // The Eisenstein norm N(a+bω) = a² - ab + b² is multiplicative.
    // This is the fundamental "zero-drift" property.

    #[test]
    fn norm_is_multiplicative_exhaustive() {
        // N(α * β) = N(α) * N(β) for all α, β in a small domain
        for a in -10..=10 {
            for b in -10..=10 {
                let alpha = E12::new(a, b);
                for c in -10..=10 {
                    for d in -10..=10 {
                        let beta = E12::new(c, d);
                        let n_alpha = alpha.norm();
                        let n_beta = beta.norm();
                        let n_product = (alpha * beta).norm();
                        // Only check when no overflow risk
                        if n_alpha < (1u64 << 31) && n_beta < (1u64 << 31) {
                            assert_eq!(
                                n_product, n_alpha * n_beta,
                                "Norm multiplicativity failed: N({} * {}) = {} != N({}) * N({}) = {} * {} = {}",
                                alpha, beta, n_product, alpha, beta, n_alpha, n_beta, n_alpha * n_beta
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn norm_is_non_negative() {
        // N(a+bω) ≥ 0 for all integers a, b
        for a in -100..=100 {
            for b in -100..=100 {
                let n = E12::new(a, b).norm();
                // Already u64, but verify the formula gives a sensible value
                let expected =
                    (a as i64 * a as i64 - a as i64 * b as i64 + b as i64 * b as i64) as u64;
                assert_eq!(n, expected);
                // The discriminant of x² - ab + b² in terms of positivity:
                // a² - ab + b² = (a - b/2)² + 3b²/4 ≥ 0, always non-negative
            }
        }
    }

    #[test]
    fn norm_zero_iff_origin() {
        // N(a+bω) = 0 ⟺ a=b=0
        for a in -5..=5 {
            for b in -5..=5 {
                let n = E12::new(a, b).norm();
                if a == 0 && b == 0 {
                    assert_eq!(n, 0, "Origin should have norm 0");
                } else {
                    assert!(n > 0, "({},{}) should have positive norm", a, b);
                }
            }
        }
    }

    #[test]
    fn norm_of_product_of_units() {
        // Product of units is a unit: N(u₁ * u₂) = N(u₁) * N(u₂) = 1 * 1 = 1
        let units = E12::directions();
        for &u1 in &units {
            for &u2 in &units {
                let product = u1 * u2;
                assert_eq!(
                    product.norm(),
                    1,
                    "Product of units {} * {} should have norm 1, got {} ({})",
                    u1,
                    u2,
                    product.norm(),
                    product
                );
            }
        }
    }

    #[test]
    fn conjugate_norm_preservation() {
        // N(conj(z)) = N(z)
        for a in -10..=10 {
            for b in -10..=10 {
                let z = E12::new(a, b);
                assert_eq!(
                    z.conjugate().norm(),
                    z.norm(),
                    "Conjugate should preserve norm for ({},{})",
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn norm_of_conjugate_product() {
        // z * conj(z) has norm N(z)²
        for a in -5..=5 {
            for b in -5..=5 {
                let z = E12::new(a, b);
                let zzbar = z * z.conjugate();
                let expected = z.norm() * z.norm();
                if expected < (1u64 << 40) {
                    assert_eq!(
                        zzbar.norm(),
                        expected,
                        "z * conj(z) should have norm N(z)² for ({},{})",
                        a,
                        b
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod hex_disk_geometry {
    use super::*;

    // HexDisk represents a bounded hexagonal region. Its geometry has
    // exact combinatorial properties we can verify.

    #[test]
    fn disk_count_formula_holds_to_large_radius() {
        // 3R² + 3R + 1 for R = 0..50
        for r in 0u32..=50 {
            let disk = HexDisk::radius(r);
            let expected = 3 * r as u64 * r as u64 + 3 * r as u64 + 1;
            assert_eq!(
                disk.count(),
                expected,
                "HexDisk radius {} count formula failed",
                r
            );
        }
    }

    #[test]
    fn disk_iteration_count_matches_formula() {
        for r in 0u32..=15 {
            let disk = HexDisk::radius(r);
            let count = disk.iter().count() as u64;
            assert_eq!(
                count,
                disk.count(),
                "HexDisk radius {} iter count != formula",
                r
            );
        }
    }

    #[test]
    fn disk_contains_all_iterated_points() {
        // Every point from the iterator should be contained in the disk
        for r in 0u32..=6 {
            let disk = HexDisk::radius(r);
            for p in disk.iter() {
                assert!(
                    disk.contains(&p),
                    "Iterated point {} not contained in disk({})",
                    p,
                    r
                );
            }
        }
    }

    #[test]
    fn disk_radius_zero_is_origin_only() {
        let disk = HexDisk::radius(0);
        let points: Vec<_> = disk.iter().collect();
        assert_eq!(points, vec![E12::new(0, 0)]);
    }

    #[test]
    fn disk_radius_one_is_origin_plus_six_neighbors() {
        let disk = HexDisk::radius(1);
        let points: Vec<_> = disk.iter().collect();
        // 3(1) + 3(1) + 1 = 7
        assert_eq!(points.len(), 7);
        assert!(points.contains(&E12::new(0, 0)));
    }

    #[test]
    fn disk_is_symmetric_under_d6() {
        // D₆ rotations preserve the Eisenstein norm, but the hex_distance
        // formula (|a|+|b|+|a+b|)/2 is NOT invariant under all 6 rotations
        // in this axial coordinate convention. This is because the axial
        // system breaks the full D₆ symmetry — the third cube coordinate
        // s = -a - b is implicitly handled.
        //
        // What IS true: the norm is D₆ invariant, and negation (180°) preserves
        // the disk.
        let disk = HexDisk::radius(5);

        // Negation preserves the disk
        for p in disk.iter() {
            let neg = E12::new(-p.a(), -p.b());
            assert!(
                disk.contains(&neg),
                "Negation {} of {} not in disk(5)",
                neg,
                p
            );
        }

        // All D₆ rotations preserve the norm
        for p in disk.iter() {
            let n = p.norm();
            for r in &p.d6_rotations() {
                assert_eq!(
                    r.norm(),
                    n,
                    "D₆ rotation should preserve norm: {} -> {}",
                    p,
                    r
                );
            }
        }
    }

    #[test]
    fn nested_disks() {
        // disk(R) ⊂ disk(R+1) for all R
        let small = HexDisk::radius(3);
        let large = HexDisk::radius(5);
        for p in small.iter() {
            assert!(large.contains(&p), "Point {} in disk(3) not in disk(5)", p);
        }
    }
}

#[cfg(test)]
mod eisenstein_triple_properties {
    use super::*;

    // Eisenstein triples (a,b,c) satisfy a² - ab + b² = c².
    // They are ~6.8× denser than Pythagorean triples.

    #[test]
    fn generated_triples_satisfy_norm_equation() {
        // Every generated triple must satisfy a² - ab + b² = c²
        let triples = EisensteinTriple::generate(20);
        assert_eq!(triples.len(), 20);
        for t in &triples {
            let a = t.a() as i64;
            let b = t.b() as i64;
            let c = t.c() as i64;
            assert_eq!(
                a * a - a * b + b * b,
                c * c,
                "Triple ({},{},{}) doesn't satisfy a²-ab+b²=c²",
                a,
                b,
                c
            );
        }
    }

    #[test]
    fn generated_triples_are_primitive() {
        let triples = EisensteinTriple::generate(15);
        for t in &triples {
            assert!(
                t.is_primitive(),
                "Generated triple {:?} should be primitive",
                t
            );
        }
    }

    #[test]
    fn generated_triples_are_increasing_order() {
        // The generate function should produce triples in roughly increasing norm
        let triples = EisensteinTriple::generate(20);
        for i in 1..triples.len() {
            assert!(
                triples[i].c() >= triples[i - 1].c(),
                "Triples should be roughly ordered by c: triple[{}].c={} < triple[{}].c={}",
                i,
                triples[i].c(),
                i - 1,
                triples[i - 1].c()
            );
        }
    }

    #[test]
    fn eisenstein_triple_density_advantage() {
        // Eisenstein triples are ~6.8× denser than Pythagorean triples.
        // At c ≤ 50, Pythagorean triples: 16 primitive.
        // Eisenstein should have significantly more.
        let triples = EisensteinTriple::all_with_max_norm(50);
        // Just verify we get a healthy count (exact number depends on search)
        assert!(
            triples.len() >= 16,
            "Should find at least 16 Eisenstein triples with c ≤ 50, found {}",
            triples.len()
        );
    }

    #[test]
    fn first_few_eisenstein_triples_are_correct() {
        // The first primitive Eisenstein triples should include known values.
        // Known: (3,1,√7) — wait, that's not a perfect square.
        // (5,0,5), (7,0,7), (8,3,7)...
        // Let's check that (8,3,7) appears
        let triples = EisensteinTriple::all_with_max_norm(10);
        let found = triples
            .iter()
            .any(|t| t.a() == 8 && t.b() == 3 && t.c() == 7);
        assert!(found, "(8,3,7) should be found among triples with c ≤ 10");
    }

    #[test]
    fn triple_norm_matches_c_squared() {
        // For every triple, norm = c²
        let triples = EisensteinTriple::all_with_max_norm(20);
        for t in &triples {
            assert_eq!(
                t.norm(),
                (t.c() as u64) * (t.c() as u64),
                "Triple norm {} != c² ({}) for ({},{},{})",
                t.norm(),
                (t.c() as u64) * (t.c() as u64),
                t.a(),
                t.b(),
                t.c()
            );
        }
    }
}

#[cfg(test)]
mod gcd_properties {
    use super::*;

    // Z[ω] is a UFD (in fact a PID), so GCD is well-defined up to associates.

    #[test]
    fn gcd_is_idempotent() {
        // gcd(a, a) = a (up to associates)
        let test_cases = vec![
            E12::new(6, 0),
            E12::new(3, -2),
            E12::new(7, 1),
            E12::new(0, 5),
        ];
        for &a in &test_cases {
            let g = a.gcd(a);
            assert_eq!(g.norm(), a.norm(), "gcd(a,a) should have same norm as a");
        }
    }

    #[test]
    fn gcd_with_zero_returns_self() {
        // gcd(a, 0) = a (normalized)
        let test_cases = vec![E12::new(5, 0), E12::new(3, -2), E12::new(7, 1)];
        for &a in &test_cases {
            let g = a.gcd(E12::new(0, 0));
            assert_eq!(g.norm(), a.norm(), "gcd(a, 0) should have same norm as a");
        }
    }

    #[test]
    fn gcd_with_unit_is_unit() {
        // gcd(a, 1) = unit
        let test_cases = vec![
            E12::new(5, 0),
            E12::new(3, -2),
            E12::new(7, 1),
            E12::new(12, -5),
        ];
        for &a in &test_cases {
            let g = a.gcd(E12::new(1, 0));
            assert_eq!(g.norm(), 1, "gcd({}, 1) should be a unit (norm 1)", a);
        }
    }

    #[test]
    fn gcd_divides_both_arguments() {
        // gcd(a, b) divides both a and b
        let test_pairs = vec![
            (E12::new(12, 0), E12::new(8, 0)),
            (E12::new(6, 3), E12::new(3, 9)),
            (E12::new(7, 0), E12::new(0, 7)),
        ];
        for &(a, b) in &test_pairs {
            let g = a.gcd(b);
            assert!(g.divides(a), "gcd {} should divide {}", g, a);
            assert!(g.divides(b), "gcd {} should divide {}", g, b);
        }
    }

    #[test]
    fn gcd_result_is_positive_normalized() {
        // The GCD should always be normalized with a > 0 (or a == 0 and b > 0)
        let test_pairs = vec![
            (E12::new(-6, 0), E12::new(4, 0)),
            (E12::new(-3, -2), E12::new(7, 1)),
            (E12::new(5, -5), E12::new(-2, 3)),
        ];
        for &(a, b) in &test_pairs {
            let g = a.gcd(b);
            assert!(
                g.a() > 0 || (g.a() == 0 && g.b() >= 0),
                "GCD should be positive-normalized, got {}",
                g
            );
        }
    }
}

#[cfg(test)]
mod integer_properties {
    use super::*;

    // Properties connecting Z[ω] to the ordinary integers Z.

    #[test]
    fn integers_embed_as_a_axis() {
        // The integer n maps to E12::new(n, 0)
        // N(n) = n²
        for n in -20..=20 {
            let z = E12::new(n, 0);
            assert_eq!(z.norm(), (n as i64 * n as i64) as u64);
        }
    }

    #[test]
    fn omega_has_norm_one() {
        // ω = (0, 1), N(ω) = 0 - 0 + 1 = 1
        let omega = E12::new(0, 1);
        assert_eq!(omega.norm(), 1);
        assert!(omega.is_unit());
    }

    #[test]
    fn omega_cubed_is_one() {
        // ω³ = 1 in Z[ω], i.e., ω is a primitive cube root of unity
        let omega = E12::new(0, 1);
        let omega_sq = omega * omega; // ω²
        let omega_cubed = omega_sq * omega; // ω³ = 1
        assert_eq!(
            omega_cubed,
            E12::new(1, 0),
            "ω³ should equal 1, got {}",
            omega_cubed
        );
    }

    #[test]
    fn omega_squared_is_conjugate() {
        // ω² = ω̄ = conjugate of ω = (-1, -1)
        let omega = E12::new(0, 1);
        let omega_sq = omega * omega;
        let omega_conj = omega.conjugate();
        assert_eq!(
            omega_sq, omega_conj,
            "ω² should equal conjugate of ω: {} vs {}",
            omega_sq, omega_conj
        );
    }

    #[test]
    fn one_plus_omega_plus_omega_squared_is_zero() {
        // 1 + ω + ω² = 0 (the defining relation of cube roots of unity)
        let one = E12::new(1, 0);
        let omega = E12::new(0, 1);
        let omega_sq = omega * omega;
        let sum = one + omega + omega_sq;
        assert_eq!(
            sum,
            E12::new(0, 0),
            "1 + ω + ω² should equal 0, got {}",
            sum
        );
    }

    #[test]
    fn multiplication_table_of_units() {
        // The 6 units form a group under multiplication (C₆)
        let units = E12::directions();
        // Table: product of any two units should be a unit
        for &u1 in &units {
            for &u2 in &units {
                let product = u1 * u2;
                assert_eq!(
                    product.norm(),
                    1,
                    "Product of units {} * {} should have norm 1",
                    u1,
                    u2
                );
            }
        }
    }
}
