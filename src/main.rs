use std::collections::HashSet;
use oxibloom::{bloom, os_random};

fn main() {
    let mut og_values: Vec<u128> = Vec::with_capacity(1_000_000);
    let mut bloom_filter = bloom::BloomFilter::new(1_000_000, 0.01);
    println!("bloom debug: {bloom_filter:?}");
    for _ in 0..1_000_000 {
        let r = os_random::get_random_u128();
        og_values.push(r.clone());
        bloom_filter.insert(r);
    }
    for v in og_values {
        if !bloom_filter.contains(v) {
            panic!("it should be contained!")
        }
    }
    println!("bloom debug #2: {bloom_filter:?}");
}
