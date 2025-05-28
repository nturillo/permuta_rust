use rayon::prelude::*;
use itertools::Itertools;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PattDetails {
    left_floor: Option<u8>,
    left_ceil: Option<u8>,
    upper_bound: u8,
    lower_bound: u8,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub perm: Perm,
    pub details: Vec<PattDetails>,
}

#[derive(Debug, Clone)]
pub struct Perm {
    pub n: u8,
    pub data: Vec<u8>,
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
    pub fn new(data: Vec<u8>) -> Self {
        let n = data.len() as u8;
        Perm {n, data}
    }
    /// Convert an index to the nth lexicographic permutation of [0, 1, ..., n-1]
    fn unrank_permutation(mut index: usize, n: u8) -> Perm {
        let mut elements: Vec<u8> = (0..n).collect();
        let mut data= Vec::with_capacity(n as usize);

        for i in (1..=(n as usize)).rev() {
            let factorial = (1..i).product::<usize>();
            let pos = index / factorial;
            index %= factorial;

            data.push(elements.remove(pos));
        }

        Perm::new(data)
    }
    pub fn par_of_length(n: u8) -> impl ParallelIterator<Item = Perm> {
        let total = (1..=(n as usize)).product::<usize>();
        (0..total)
            .into_par_iter()
            .map(move |i| Perm::unrank_permutation(i, n))
    }
    pub fn of_length(n: u8) -> impl Iterator<Item = Perm> {
        (0..n).permutations(n as usize).map(Perm::new)
    }

    pub fn occurences_of(&self, patt: &Pattern) -> Vec<Vec<u8>> {
        patt.occurrences_in(self)
    }


    pub fn count_occurrences_of(&self, patt: &Pattern) -> usize {
        patt.count_occurrences_in(self)
    }

}

impl Pattern {
    pub fn new(perm: Perm) -> Self {
        let details = perm.data 
            .iter()
            .copied()
            .zip(Pattern::left_floor_and_ceil(&perm))
            .map(|(val, (floor, ceiling))| {
                let left_diff = match floor {
                    Some(f) => val - perm.data[f as usize],
                    None => val,
                };
                let right_diff = match ceiling {
                    Some(c) => perm.data[c as usize] - val,
                    None => perm.n - val,
                };
            
                PattDetails {
                    left_floor: floor,
                    left_ceil: ceiling,
                    upper_bound: right_diff,
                    lower_bound: left_diff,
                }
            })
            .collect();
        Pattern {
            perm,
            details
        }
    }
    fn left_floor_and_ceil(perm: &Perm) -> Vec<(Option<u8>, Option<u8>)> {
        // For each element, return the pair of indices of (largest less, smalllest
        // greater) to the left, if they exist. If not, -1 is used instead.
        let mut deq: VecDeque<(u8, u8)> = VecDeque::new(); // (value, index)
        let mut results = Vec::new();

        let mut smallest: u8 = 0;
        let mut biggest: u8 = 0;
        for (idx, &val) in perm.data.iter().enumerate() {
            if idx == 0 {
                deq.push_back((val, idx as u8));
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
                deq.push_front((val, idx as u8));
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
                deq.push_back((val, idx as u8));
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
                deq.push_front((val, idx as u8));
            }
        }
        results
    }

    /// Generate all patterns of length `n` from the given permutation.
    pub fn of_length(n: u8) -> impl Iterator<Item = Pattern> {
        Perm::of_length(n)
            .map(Pattern::new)
    }

    /// Generate all patterns of length `n` from the given permutation in parallel.
    pub fn par_of_length(n: u8) -> impl ParallelIterator<Item = Pattern> {
        Perm::par_of_length(n)
            .map(Pattern::new)
    }

    /// Finds all index tuples where `self` occurs in `patt`.
    pub fn occurrences_in(&self, perm: &Perm) -> Vec<Vec<u8>> {
        let n = self.perm.n;
        let m = perm.n;
        let perm_data= perm.data.as_slice();

        let mut results = Vec::new();

        if n == 0 {
            return results;
        }

        if n > m {
            return results;
        }

        // Use precomputed pattern details (floors, ceilings, precomputed diffs)
        let pattern_details = &self.details;// Returns Vec<PattDetails>

        // Preallocate the occurrence vector
        let mut occ_indices: Vec<u8>= vec![0; n as usize];

        // Each stack item is (i: pattern index, k: self occ position)
        fn occurrences(mut i: u8, k: u8, m: u8, n: u8, 
            pattern_details: &Vec<PattDetails>, pattern: &[u8], occ_indices: &mut Vec<u8>,
            res: &mut Vec<Vec<u8>>) {

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
            } = pattern_details[k as usize];

            // Compute bounds for this level
            let lo = match left_floor {
                Some(lfi) => pattern[occ_indices[lfi as usize] as usize] + lower_bound,
                None => lower_bound,
            };

            let hi = match left_ceil {
                Some(lci) => pattern[occ_indices[lci as usize] as usize] - upper_bound,
                None => m - upper_bound,
            };

            // Scan forward from i
            loop {
                if elements_remaining < elements_needed {
                    return;
                }

                let val = pattern[i as usize];
                if lo <= val && val <= hi {
                    occ_indices[k as usize] = i;

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
        occurrences(0, 0, m, n, pattern_details, perm_data, &mut occ_indices, &mut results);

        results
    }

    pub fn count_occurrences_in(&self, perm: &Perm) -> usize {
        let n = self.perm.n;
        let m = perm.n;
        let perm_data = perm.data.as_slice();

        let mut res: usize = 0;

        if n == 0 {
            return res;
        }

        if n > m {
            return res;
        }

        // Use precomputed pattern details (floors, ceilings, precomputed diffs)
        let pattern_details = &self.details; // Returns Vec<PattDetails>

        // Preallocate the occurrence vector
        let mut occ_indices: Vec<u8>= vec![0; n as usize];

        // Each stack item is (i: pattern index, k: self occ position)
        fn occurrences(mut i: u8, k: u8, m: u8, n: u8, 
            pattern_details: &Vec<PattDetails>, pattern: &[u8], occ_indices: &mut Vec<u8>,
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
            } = pattern_details[k as usize];

            // Compute bounds for this level
            let lo = match left_floor {
                Some(lfi) => pattern[occ_indices[lfi as usize] as usize] + lower_bound,
                None => lower_bound,
            };

            let hi = match left_ceil {
                Some(lci) => pattern[occ_indices[lci as usize] as usize] - upper_bound,
                None => m - upper_bound,
            };

            // Scan forward from i
            loop {
                if elements_remaining < elements_needed {
                    return;
                }

                let val = pattern[i as usize];
                if lo <= val && val <= hi {
                    occ_indices[k as usize] = i;

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
        occurrences(0, 0, m, n, pattern_details, perm_data, &mut occ_indices, &mut res);
        res
    }

    /// Counts the number of permutations of length n which have odd many
    /// occurrences of `self` as a pattern and those with even many occurrences.
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
    pub fn count_odd_even_occurrences(&self, n: u8) -> (usize, usize) {
        let even_count = Perm::par_of_length(n)
            .filter(|perm| self.count_occurrences_in(perm) % 2 == 0)
            .count();
        let total = (1..=(n as usize)).product::<usize>();
        let odd_count = total - even_count;

        (even_count, odd_count)
    }
}

impl fmt::Display for Pattern{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data_str = self.perm.data
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "Pattern (n = {}): [{}]", self.perm.n, data_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occurrences_in() {
        let pattern= Pattern::new(Perm::new(vec![2, 0, 1]));
        let perm= Perm::new(vec![5, 3, 0, 4, 2, 1]);
        let occurrences = pattern.occurrences_in(&perm);
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