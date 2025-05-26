use permuta_rust::Perm;

fn main() {
    let n = 3;
    let m = 10;

    let (even_count, odd_count) = Perm::new(vec![2, 0, 1]).count_odd_even_occurrences(11);
    println!("For n = {}, m = {}: {} even, {} odd", n, m, even_count, odd_count);
    return;

    for perm in Perm::of_length(n) {
        println!("Perm: {}", perm);
        for j in n..=m {
            let (even_count, odd_count) = perm.count_odd_even_occurrences(j);
            println!("j = {}: {} even, {} odd", j, even_count, odd_count);
        }
    }
}