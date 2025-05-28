use permuta_rust::{Perm, Pattern};
use std::time::Instant;

fn main() {
    let n = 3;
    let m = 12;


    /*
    for perm in Perm::of_length(n) {
        println!("Perm: {}", perm);
        for j in n..=m {
            let (even_count, odd_count) = perm.count_odd_even_occurrences(j);
            println!("j = {}: {} even, {} odd", j, even_count, odd_count);
        }
    }
    */
    let patt= Pattern::of_length(n).next().unwrap();
    println!("Pattern: {}", patt);
    for j in n..=m {
        let start = Instant::now();
        let (even_count, odd_count) = patt.count_odd_even_occurrences(j);
        println!("j = {}: {} even, {} odd", j, even_count, odd_count);
        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);
    }

}