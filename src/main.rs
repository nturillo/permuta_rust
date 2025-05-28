use permuta_rust::Perm;
use std::time::Instant;

fn main() {
    let n = 5;
    let m = 11;

    let start = Instant::now();

    /*
    for perm in Perm::of_length(n) {
        println!("Perm: {}", perm);
        for j in n..=m {
            let (even_count, odd_count) = perm.count_odd_even_occurrences(j);
            println!("j = {}: {} even, {} odd", j, even_count, odd_count);
        }
    }
    */
    let perm = Perm::of_length(n).next().unwrap();
    println!("Perm: {}", perm);
    for j in n..=m {
        let (even_count, odd_count) = perm.count_odd_even_occurrences(j);
        println!("j = {}: {} even, {} odd", j, even_count, odd_count);
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}