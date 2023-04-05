use std::collections::HashSet;
use oxibloom::{bloom, random};

fn main() {
    let mut og_values: Vec<u128> = Vec::with_capacity(1_000_000);
    let mut bloom_filter = bloom::BloomFilter::new(1_000_000, 0.99);
    for _ in 0..1_000_000 {
        let r = random::get_random_u128();
        og_values.push(r.clone());
        bloom_filter.insert(r);
    }
    for v in og_values {
        if !bloom_filter.contains(v) {
            panic!("it should be contained!")
        }
    }
}
