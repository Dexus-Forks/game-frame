//! The Microsoft C runtime's `qsort`: an unstable quicksort with a median-of-three pivot and a
//! selection sort for partitions of 8 elements or fewer. Elements that compare equal come out
//! in an order that depends on the algorithm. A game that sorts with `qsort` makes that order
//! part of its behaviour, so a port must sort the same way.
//!
//! ```
//! use gameframe_msvc_crt::qsort::qsort_by;
//!
//! let mut v = vec![(3, 'a'), (1, 'b'), (3, 'c'), (2, 'd')];
//! qsort_by(&mut v, |x, y| x.0 - y.0);
//! assert_eq!(v.iter().map(|p| p.0).collect::<Vec<_>>(), [1, 2, 3, 3]);
//! ```
//!
//! Original: MSVC `qsort` (in Banished's executable, `FUN_140030770` and its templated copies).
//! In Un-Banished, the order of equal elements matches the original on recorded sorts
//! (*measured*).

/// The Microsoft C runtime's `qsort` with a `less` comparator, as the game's templated copies use
/// it: the same algorithm as [`qsort_by`], with `cmp(a, b) > 0` being `less(b, a)` and
/// equality neither way.
///
/// Original: MSVC `qsort` called through a C++ `less` adapter (Banished has eight such copies).
pub fn qsort_less<T>(a: &mut [T], less: impl Fn(&T, &T) -> bool) {
    qsort_by(a, |x, y| {
        if less(y, x) {
            1
        } else if less(x, y) {
            -1
        } else {
            0
        }
    });
}

/// The same with a three-way comparator.
pub fn qsort_by<T>(a: &mut [T], cmp: impl Fn(&T, &T) -> i32) {
    const CUTOFF: usize = 8;
    if a.len() < 2 {
        return;
    }
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let (mut lo, mut hi) = (0usize, a.len() - 1);
    loop {
        let size = hi - lo + 1;
        if size <= CUTOFF {
            // shortsort
            let mut h = hi;
            while h > lo {
                let mut max = lo;
                for p in lo + 1..=h {
                    if cmp(&a[p], &a[max]) > 0 {
                        max = p;
                    }
                }
                a.swap(max, h);
                h -= 1;
            }
        } else {
            let mut mid = lo + size / 2;
            if cmp(&a[lo], &a[mid]) > 0 {
                a.swap(lo, mid);
            }
            if cmp(&a[lo], &a[hi]) > 0 {
                a.swap(lo, hi);
            }
            if cmp(&a[mid], &a[hi]) > 0 {
                a.swap(mid, hi);
            }
            let (mut loguy, mut higuy) = (lo as isize, hi as isize);
            let m = |mid: usize| mid as isize;
            loop {
                if m(mid) > loguy {
                    loop {
                        loguy += 1;
                        if !(loguy < m(mid) && cmp(&a[loguy as usize], &a[mid]) <= 0) {
                            break;
                        }
                    }
                }
                if m(mid) <= loguy {
                    loop {
                        loguy += 1;
                        if !(loguy <= hi as isize && cmp(&a[loguy as usize], &a[mid]) <= 0) {
                            break;
                        }
                    }
                }
                loop {
                    higuy -= 1;
                    if !(higuy > m(mid) && cmp(&a[higuy as usize], &a[mid]) > 0) {
                        break;
                    }
                }
                if higuy < loguy {
                    break;
                }
                a.swap(loguy as usize, higuy as usize);
                if m(mid) == higuy {
                    mid = loguy as usize;
                }
            }
            higuy += 1;
            if m(mid) < higuy {
                loop {
                    higuy -= 1;
                    if !(higuy > m(mid) && cmp(&a[higuy as usize], &a[mid]) == 0) {
                        break;
                    }
                }
            }
            if m(mid) >= higuy {
                loop {
                    higuy -= 1;
                    if !(higuy > lo as isize && cmp(&a[higuy as usize], &a[mid]) == 0) {
                        break;
                    }
                }
            }
            if higuy - lo as isize >= hi as isize - loguy {
                if (lo as isize) < higuy {
                    stack.push((lo, higuy as usize));
                }
                if loguy < hi as isize {
                    lo = loguy as usize;
                    continue;
                }
            } else {
                if loguy < hi as isize {
                    stack.push((loguy as usize, hi));
                }
                if (lo as isize) < higuy {
                    hi = higuy as usize;
                    continue;
                }
            }
        }
        match stack.pop() {
            Some((l, h)) => {
                lo = l;
                hi = h;
            }
            None => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small linear congruential generator for test data.
    fn data(n: usize, seed: u32, range: i32) -> Vec<(i32, usize)> {
        let mut s = seed;
        (0..n)
            .map(|i| {
                s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                (((s >> 16) as i32).rem_euclid(range), i)
            })
            .collect()
    }

    #[test]
    fn sorts() {
        for n in [0, 1, 2, 7, 8, 9, 50, 1000] {
            let mut v = data(n, n as u32 + 1, 10);
            qsort_by(&mut v, |a, b| a.0 - b.0);
            assert!(v.windows(2).all(|w| w[0].0 <= w[1].0), "n = {n}");
            let mut w = data(n, n as u32 + 1, 10);
            qsort_less(&mut w, |a, b| a.0 < b.0);
            assert_eq!(v, w, "the two entry points sort the same way");
        }
    }

    #[test]
    fn is_not_stable() {
        let mut v = data(100, 7, 3);
        qsort_by(&mut v, |a, b| a.0 - b.0);
        let stable = v.windows(2).all(|w| w[0].0 != w[1].0 || w[0].1 < w[1].1);
        assert!(!stable, "equal keys keep their input order, which MSVC qsort does not");
    }
}
