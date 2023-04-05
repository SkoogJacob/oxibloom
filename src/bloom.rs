use std::collections::hash_map::{DefaultHasher, RandomState};
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use core::f64::consts::LN_2;

pub struct BloomFilter<T>
    where T: ?Sized {
    bitmap: Vec<u8>,
    optimal_m: u64,
    byte_count: usize,
    optimal_k: u32,
    hashers: [DefaultHasher; 2],
    _marker: PhantomData<T>
}

struct ByteIndex(usize, u8);

impl<T> BloomFilter<T> {
    const FF_BYTE: u8 = 0xff;

    pub fn new(item_count: usize, fp_rate: f64) -> Self
    where T: Default {
        let optimal_map_size = Self::bitmap_size(item_count, fp_rate);
        let optimal_hasher_count = Self::hasher_count(fp_rate);
        let hashers = [
            RandomState::new().build_hasher(),
            RandomState::new().build_hasher()
        ];
        let byte_count = optimal_map_size / 8;
        let mut bitmap: Vec<u8> = Vec::with_capacity(byte_count);
        bitmap.fill(0);
        BloomFilter {
            bitmap,
            optimal_m: optimal_map_size as u64,
            byte_count,
            optimal_k: optimal_hasher_count,
            hashers,
            _marker: PhantomData
        }
    }

    pub fn insert(&mut self, elem: T) where T: Hash {
        let (base_hash_1, base_hash_2) = self.hash_kernel(elem);

        for i in 0..self.optimal_k {
            let index = self.get_index(base_hash_1, base_hash_2, i as u64);
            self.set_byte(index);
        }
    }

    pub fn contains(&self, elem: T) -> bool where T: Hash {
        let (base_hash_1, base_hash_2) = self.hash_kernel(elem);

        for i in 0..self.optimal_k {
            let index = self.get_index(base_hash_1, base_hash_2, i as u64);
            if !self.bit_is_set(index) {
                return false;
            }
        }
        true
    }

    fn bitmap_size(item_count: usize, fp_rate: f64) -> usize {
        let ln_2_squared = LN_2 * LN_2;
        ((-1.0f64 * item_count as f64 * fp_rate.ln()) / ln_2_squared).ceil() as usize
    }
    fn hasher_count(fp_rate: f64) -> u32 {
        ((-1.0f64 * fp_rate.ln()) / LN_2).ceil() as u32
    }
    fn hash_kernel(&self, elem: T) -> (u64, u64) where T: Hash {
        let (hasher1, hasher2) = (&mut self.hashers[0].clone(), &mut self.hashers[1].clone());
        elem.hash(hasher1);
        elem.hash(hasher2);

        (hasher1.finish(), hasher2.finish())
    }

    fn get_index(&self, hash1: u64, hash2: u64, hash_set_number: u64) -> ByteIndex {
        fn get_byte_bit_index(num: u64) -> (u64, u8) {
            (num / 8, (num % 8) as u8)
        }
        let (byte, bit) = get_byte_bit_index(hash1.wrapping_add(hash_set_number.wrapping_mul(hash2)));
        return ByteIndex(byte as usize, bit)
    }

    fn get_bitmask(bit_index: u8) -> u8 {
        match bit_index {
            0 => 0b10000000,
            1 => 0b01000000,
            2 => 0b00100000,
            3 => 0b00010000,
            4 => 0b00001000,
            5 => 0b00000100,
            6 => 0b00000010,
            7 => 0b00000001,
            _ => 0
        }
    }

    fn set_byte(&mut self, byte_index: ByteIndex) {
        let bitmask = Self::get_bitmask(byte_index.1);
        let byte = self.bitmap.get_mut(byte_index.0 % self.byte_count).unwrap();
        *byte = *byte | bitmask;
    }

    fn bit_is_set(&self, byte_index: ByteIndex) -> bool {
        let bitmask = Self::get_bitmask(byte_index.1);
        let byte: &u8 = self.bitmap.get(byte_index.0 % self.byte_count).unwrap();
        // if the byte contains the bit this XOR will be smaller than the original byte
        (*byte ^ bitmask) < *byte
    }
}