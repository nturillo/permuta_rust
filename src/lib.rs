use rayon::prelude::*;
use itertools::Itertools;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PattDetails {
    left_floor: Option<usize>,
    left_ceil: Option<usize>,
    upper_bound: usize,
    lower_bound: usize,
}

#[derive(Debug, Clone)]
pub struct Perm {
    pub n: usize,
    pub data: Vec<usize>,
    
    pattern_details: Option<Vec<PattDetails>>,
}


use std::fmt;

impl fmt::Display for Perm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data_str = self.data
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "Perm (n = {}): [{}]", self.n, data_str)
    }
}

impl Perm {
    pub fn new(data: Vec<usize>) -> Self {
        let n = data.len();
        let mut res = Perm { n, data, pattern_details: None };
        res.set_pattern_details();
        res
    }
    pub fn of_length(n: usize) -> impl Iterator<Item = Perm> {
        (0..n).permutations(n).map(Perm::new)
    }
    pub fn left_floor_and_ceil(&self) -> Vec<(Option<usize>, Option<usize>)> {
        // For each element, return the pair of indices of (largest less, smalllest
        // greater) to the left, if they exist. If not, -1 is used instead.
        let mut deq: VecDeque<(usize, usize)> = VecDeque::new(); // (value, index)
        let mut results = Vec::new();

        let mut smallest: usize = 0;
        let mut biggest: usize = 0;
        for (idx, &val) in self.data.iter().enumerate() {
            if idx == 0 {
                deq.push_back((val, idx));
                smallest = val;
                biggest = val;
                results.push((None, None));
            } else if val < smallest {
                // Rotate until smallest value is at front
                while let Some(&(v, _)) = deq.front() {
                    if v == smallest {
                        break;
                    }
                    if let Some(back) = deq.pop_back() {
                        deq.push_front(back);
                    }
                }
                results.push((None, Some(deq.front().unwrap().1)));
                deq.push_front((val, idx));
                smallest = val;
            } else if val > biggest {
                // Rotate until biggest value is at back
                while let Some(&(v, _)) = deq.back() {
                    if v == biggest {
                        break;
                    }
                    if let Some(front) = deq.pop_front() {
                        deq.push_back(front);
                    }
                }
                results.push((Some(deq.back().unwrap().1), None));
                deq.push_back((val, idx));
                biggest = val;
            } else {
                while !(deq.back().unwrap().0 <= val && val <= deq.front().unwrap().0) {
                    if let Some(front) = deq.pop_front() {
                        deq.push_back(front);
                    }
                }
                results.push((
                    Some(deq.back().unwrap().1),
                    Some(deq.front().unwrap().1),
                ));
                deq.push_front((val, idx));
            }
        }

        results
    }

    pub fn set_pattern_details(&mut self) {
        if self.pattern_details.is_none() {
            self.pattern_details = Some(self.data 
            .iter()
            .copied()
            .zip(self.left_floor_and_ceil())
            .map(|(val, (floor, ceiling))| {
                let left_diff = match floor {
                    Some(f) => val - self.data[f],
                    None => val,
                };
                let right_diff = match ceiling {
                    Some(c) => self.data[c] - val,
                    None => self.n - val,
                };

                PattDetails {
                    left_floor: floor,
                    left_ceil: ceiling,
                    upper_bound: right_diff,
                    lower_bound: left_diff,
                }
            })
            .collect());
        }
    }

    pub fn get_pattern_details(&self) -> &Vec<PattDetails> {
        self.pattern_details.as_ref().expect("Pattern details not set")
    }

    /// Finds all index tuples where `self` occurs in `patt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use permuta_rust::Perm;
    /// let perm = Perm::new(vec![2, 0, 1]);
    /// let pattern = Perm::new(vec![5, 3, 0, 4, 2, 1]);
    /// let occurrences = perm.occurrences_in(&pattern);
    /// assert_eq!(occurrences, vec![
    ///     vec![0, 1, 3],
    ///     vec![0, 2, 3],
    ///     vec![0, 2, 4],
    ///     vec![0, 2, 5],
    ///     vec![1, 2, 4],
    ///     vec![1, 2, 5],
    /// ]);
    /// ```
    pub fn occurrences_in(&self, patt: &Perm) -> Vec<Vec<usize>> {
        let n = self.n;
        let m = patt.n;
        let pattern = patt.data.as_slice();

        let mut results = Vec::new();

        if n == 0 {
            results.push(Vec::new());
            return results;
        }

        if n > m {
            return results;
        }

        // Use precomputed pattern details (floors, ceilings, precomputed diffs)
        let pattern_details = self.get_pattern_details(); // Returns Vec<PattDetails>

        // Preallocate the occurrence vector
        let mut occ_indices: Vec<usize>= vec![0; n];

        // Each stack item is (i: pattern index, k: self occ position)
        fn occurrences(mut i: usize, k: usize, m: usize, n: usize, 
            pattern_details: &Vec<PattDetails>, pattern: &[usize], occ_indices: &mut Vec<usize>,
            res: &mut Vec<Vec<usize>>) {

            let mut elements_remaining = m - i;
            let elements_needed = n - k;

            if elements_remaining < elements_needed {
                return; // Not enough elements left to fill the pattern
            }

            let PattDetails {
                left_floor,
                left_ceil,
                lower_bound,
                upper_bound,
            } = pattern_details[k];

            // Compute bounds for this level
            let lo = match left_floor {
                Some(lfi) => pattern[occ_indices[lfi]] + lower_bound,
                None => lower_bound,
            };

            let hi = match left_ceil {
                Some(lci) => pattern[occ_indices[lci]] - upper_bound,
                None => m - upper_bound,
            };

            // Scan forward from i
            loop {
                if elements_remaining < elements_needed {
                    return;
                }

                let val = pattern[i];
                if lo <= val && val <= hi {
                    occ_indices[k] = i;

                    if k == n - 1 {
                        res.push(occ_indices.clone()); // Only clone when full match is found
                    } else {
                        occurrences(i+1, k+1, m, n, pattern_details, pattern, occ_indices, res);
                    }
                }
                i += 1;
                elements_remaining -= 1;
            }
        }

        // Start the recursive search
        occurrences(0, 0, m, n, pattern_details, pattern, &mut occ_indices, &mut results);

        results
    }
    pub fn occurences_of(&self, patt: &Perm) -> Vec<Vec<usize>> {
        patt.occurrences_in(self)
    }
    pub fn count_occurrences_in(&self, patt: &Perm) -> usize {
        let n = self.n;
        let m = patt.n;
        let pattern = patt.data.as_slice();

        let mut res: usize = 0;

        if n == 0 {
            return res;
        }

        if n > m {
            return res;
        }

        // Use precomputed pattern details (floors, ceilings, precomputed diffs)
        let pattern_details = self.get_pattern_details(); // Returns Vec<PattDetails>

        // Preallocate the occurrence vector
        let mut occ_indices: Vec<usize>= vec![0; n];

        // Each stack item is (i: pattern index, k: self occ position)
        fn occurrences(mut i: usize, k: usize, m: usize, n: usize, 
            pattern_details: &Vec<PattDetails>, pattern: &[usize], occ_indices: &mut Vec<usize>,
            res: &mut usize) {

            let mut elements_remaining = m - i;
            let elements_needed = n - k;

            if elements_remaining < elements_needed {
                return; // Not enough elements left to fill the pattern
            }

            let PattDetails {
                left_floor,
                left_ceil,
                lower_bound,
                upper_bound,
            } = pattern_details[k];

            // Compute bounds for this level
            let lo = match left_floor {
                Some(lfi) => pattern[occ_indices[lfi]] + lower_bound,
                None => lower_bound,
            };

            let hi = match left_ceil {
                Some(lci) => pattern[occ_indices[lci]] - upper_bound,
                None => m - upper_bound,
            };

            // Scan forward from i
            loop {
                if elements_remaining < elements_needed {
                    return;
                }

                let val = pattern[i];
                if lo <= val && val <= hi {
                    occ_indices[k] = i;

                    if k == n - 1 {
                        *res += 1; // Only clone when full match is found
                    } else {
                        occurrences(i+1, k+1, m, n, pattern_details, pattern, occ_indices, res);
                    }
                }
                i += 1;
                elements_remaining -= 1;
            }
        }

        // Start the recursive search
        occurrences(0, 0, m, n, pattern_details, pattern, &mut occ_indices, &mut res);
        res
    }

    pub fn count_occurrences_of(&self, patt: &Perm) -> usize {
        patt.count_occurrences_in(self)
    }
    /// Counts the number of permutations of length n which have odd many
    /// occurrences of `self` as a pattern, and those with even many occurrences.
    /// 
    /// # Examples
    ///
    /// ```
    /// use permuta_rust::Perm;
    /// let perm = Perm::new(vec![2, 0, 1]);
    /// let (even_count, odd_count) = perm.count_odd_even_occurrences(5);
    /// assert_eq!(even_count, 80);
    /// assert_eq!(odd_count, 40);
    /// ```
    pub fn count_odd_even_occurrences(&self, n: usize) -> (usize, usize) {
        let perms: Vec<_> = Perm::of_length(n).collect();
        let even_count = perms
            .par_iter()
            .filter(|perm| self.count_occurrences_in(perm) % 2 == 0)
            .count();
        let odd_count = perms.len() - even_count;

        (even_count, odd_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occurrences_in() {
        let perm = Perm::new(vec![2, 0, 1]);
        let pattern = Perm::new(vec![5, 3, 0, 4, 2, 1]);
        let occurrences = perm.occurrences_in(&pattern);
        assert_eq!(occurrences, vec![
            vec![0, 1, 3],
            vec![0, 2, 3],
            vec![0, 2, 4],
            vec![0, 2, 5],
            vec![1, 2, 4],
            vec![1, 2, 5],
        ]);
    }
}