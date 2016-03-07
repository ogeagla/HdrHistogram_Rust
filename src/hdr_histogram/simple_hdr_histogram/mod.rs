use std::cmp;
use std::cmp::Ord;
use num::traits::Zero;
use num::traits::One;
use num::traits::ToPrimitive;

mod iterator;
mod test;

/// Marker trait for types we allow (namely, u8-u64)
pub trait HistogramCount : Ord + Zero + One + ToPrimitive + Copy {}

impl HistogramCount for u8 {}
impl HistogramCount for u16 {}
impl HistogramCount for u32 {}
impl HistogramCount for u64 {}

///
/// This struct essentially encapsulates the "instance variables" of the histogram
///
#[derive(Debug)]
pub struct SimpleHdrHistogram<T:HistogramCount> {
    leading_zeros_count_base: usize,
    sub_bucket_mask: u64,
    unit_magnitude: u32,
    sub_bucket_count: usize,
    sub_bucket_half_count: usize,
    sub_bucket_half_count_magnitude: u32,
    counts: Vec<T>,
    counts_array_length: usize,
    normalizing_index_offset: usize,
    max_value: u64,
    min_non_zero_value: u64,
    unit_magnitude_mask: u64,
    total_count: u64,
}

pub trait HistogramBase<T: HistogramCount> {

    //FIXME this stuff could be mostly unsigned

    //TODO this block should be default impl of this trait
    fn record_single_value(&mut self, value: u64) -> Result<(), String>;
    fn counts_array_index(&self, value: u64) -> usize;
    // in the Java impl, the functions above/below have same name but are overloaded, which
    //  Rust does not allow, thus the name change
    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize;
    fn get_bucket_index(&self, value: u64) -> usize;
    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize;
    fn get_count(&self) -> u64;
    fn get_max(&self) -> u64;
    fn update_min_and_max(&mut self, value: u64);
    fn update_max_value(&mut self, value: u64);
    fn update_min_non_zero_value(&mut self, value: u64);
    fn get_count_at_value(&self, value: u64) -> Result<T, String>;
    fn get_value_at_percentile(&self, percentile: f64) -> u64;
    fn value_from_index(&self, index: usize) -> u64;
    // in the Java impl, the functions above/below have same name but are overloaded, which
    //  Rust does not allow, thus the name change
    fn value_from_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> u64;
    fn lowest_equivalent_value(&self, value: u64) -> u64;
    fn highest_equivalent_value(&self, value: u64) -> u64;
    fn next_non_equivalent_value(&self, value: u64) -> u64;
    fn size_of_equivalent_value_range(&self, value: u64) -> u64;
    fn get_mean(&self) -> f64;
    // end TODO

    fn increment_count_at_index(&mut self, index: usize) -> Result<(), String>;
    fn normalize_index(&self, index: usize, normalizing_index_offset: usize, array_length: usize) ->
        Result<usize, String>;
    fn increment_total_count(&mut self);
    fn get_count_at_index(&self, index: usize) -> Result<T, String>;

}

impl<T: HistogramCount> HistogramBase<T> for SimpleHdrHistogram<T> {

    fn get_mean(&self) -> f64 {

        if self.get_count() == 0 { 0.0 } else {
            //TODO stuff
            0.0
        }
    }

    fn next_non_equivalent_value(&self, value: u64) -> u64 {
        self.lowest_equivalent_value(value) + self.size_of_equivalent_value_range(value)
    }

    fn size_of_equivalent_value_range(&self, value: u64) -> u64 {
        let bucket_index = self.get_bucket_index(value);
        let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
        let distance_to_next_value =
            1 << (self.unit_magnitude
                    + bucket_index as u32
                    + if sub_bucket_index >= self.sub_bucket_count {1} else {0});
        distance_to_next_value as u64
    }

    fn highest_equivalent_value(&self, value: u64) -> u64 {
        self.next_non_equivalent_value(value) - 1
    }

    fn lowest_equivalent_value(&self, value: u64) -> u64 {
        let bucket_index = self.get_bucket_index(value);
        let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
        let this_value_base_level = self.value_from_index_sub(bucket_index, sub_bucket_index);
        this_value_base_level
    }

    fn value_from_index(&self, index: usize) -> u64 {
        let mut bucket_index = (index as u32 >> self.sub_bucket_half_count_magnitude) - 1;
        let mut sub_bucket_index = (index as u32 & (self.sub_bucket_half_count as u32 - 1)) + self.sub_bucket_half_count as u32;
        if bucket_index < 0 {
            sub_bucket_index -= self.sub_bucket_half_count as u32;
            bucket_index = 0;
        }
        self.value_from_index_sub(bucket_index as usize, sub_bucket_index as usize)
    }

    fn value_from_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> u64 {
        (sub_bucket_index as u64) << (bucket_index as u32 + self.unit_magnitude)
    }

    fn get_value_at_percentile(&self, percentile: f64) -> u64 {
        let requested_percentile = percentile.min(100.0);
        let mut count_at_percentile = (((requested_percentile / 100.0) * self.get_count() as f64) + 0.5) as u64;
        count_at_percentile = cmp::max(count_at_percentile, 1);
        let mut total_to_current_index: u64 = 0;
        for i in 0..self.counts_array_length {
            let count_at_index = self.get_count_at_index(i as usize);
            match count_at_index {
                Ok(the_index) => {
                    // we only use u8 - u64 types, so this must always work
                    total_to_current_index += the_index.to_u64().unwrap();
                    if total_to_current_index >= count_at_percentile {
                        let value_at_index = self.value_from_index(i as usize);
                        return if percentile == 0.0 {
                            self.lowest_equivalent_value(value_at_index)
                        } else {
                            self.highest_equivalent_value(value_at_index)
                        }
                    }
                }
                Err(_) => { return 0 }
            }
        }
        0
    }

    fn get_count_at_index(&self, index: usize) -> Result<T, String> {
        let normalized_index =
            self.normalize_index(index, self.normalizing_index_offset, self.counts_array_length);
        match normalized_index {
            Ok(the_index) =>
                Ok(self.counts[the_index]),
            Err(err) =>
                Err(err)
        }
    }

    fn get_count_at_value(&self, value: u64) -> Result<T, String> {
        let index = cmp::min(cmp::max(0, self.counts_array_index(value)), self.counts_array_length - 1);
        self.get_count_at_index(index)
    }

    fn increment_total_count(&mut self) {
        self.total_count += 1;
    }

    fn update_max_value(&mut self, value: u64) {
        let internal_value = value | self.unit_magnitude_mask;
        self.max_value = internal_value;
    }

    fn update_min_non_zero_value(&mut self, value: u64) {
        if value <= self.unit_magnitude_mask {
            return
        }
        let internal_value = value & !self.unit_magnitude_mask;
        self.min_non_zero_value = internal_value;
    }

    fn update_min_and_max(&mut self, value: u64) {
        if value > self.max_value {
            self.update_max_value(value);
        }
        if value < self.min_non_zero_value && value != 0 {
            self.update_min_non_zero_value(value);
        }
    }

    fn normalize_index(&self, index: usize, normalizing_index_offset: usize, array_length: usize) ->
        Result<usize, String> {
        match normalizing_index_offset {
            0 => Ok(index),
            _ =>
                if index > array_length {
                    Err(String::from("index out of covered range"))
                } else {
                    let mut normalized_index: usize = index - normalizing_index_offset;
                    if normalized_index >= array_length {
                        normalized_index -=array_length;
                    }
                    Ok(normalized_index)
                }
        }
    }

    fn increment_count_at_index(&mut self, index: usize) -> Result<(), String> {
        let normalized_index =
            self.normalize_index(index, self.normalizing_index_offset, self.counts_array_length);
        match normalized_index {
            Ok(the_index) => {
                self.counts[the_index] = self.counts[the_index] + T::one();
                Ok(())
            }
            Err(err) =>
                Err(err)
        }
    }

    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize {
        // safe cast: sub bucket indexes are at most 2 * 10^precision, so can fit in usize.
        (value >> (bucket_index as u32 + self.unit_magnitude)) as usize
    }

    fn get_bucket_index(&self, value: u64) -> usize {
        let value_orred = value | self.sub_bucket_mask;
        self.leading_zeros_count_base - (value_orred.leading_zeros() as usize)
    }

    fn record_single_value(&mut self, value: u64) -> Result<(), String> {
        let counts_index = self.counts_array_index(value);
            match self.increment_count_at_index(counts_index) {
                Ok(_) => {
                    self.update_min_and_max(value);
                    self.increment_total_count();
                    Ok(())
                }
                Err(err) => {
                    Err(String::from(format!("Could not increment count at index due to: {}", err)))
                }
            }
    }

    fn get_max(&self) -> u64 {
        self.max_value
    }

    fn get_count(&self) -> u64 {
        self.total_count
    }

    fn counts_array_index(&self, value: u64) -> usize {
        let bucket_index = self.get_bucket_index(value);
        let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
        return self.counts_array_index_sub(bucket_index, sub_bucket_index);
    }

    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize {
        assert!(sub_bucket_index < self.sub_bucket_count);
        assert!(bucket_index == 0 || (sub_bucket_index >= self.sub_bucket_half_count));

        let bucket_base_index = (bucket_index + 1) << self.sub_bucket_half_count_magnitude;

        // offset_in_bucket can be negative for bucket 0.
        // these casts are safe: sub_bucket_index is at most sub_bucket_count, and sub_bucket_count
        // is at most 2 * 10^precision.
        let offset_in_bucket: i32 = sub_bucket_index as i32 - self.sub_bucket_half_count as i32;

        // buckets scale with 2^x * (sub bucket count), so bucket index could be at most the bit
        // length of the value datatype (e.g. 64 bits), and since sub bucket count is > 1 in
        // practice, it's even smaller. Thus, this case to signed is safe.

        let bucket_base_signed: i32 = bucket_base_index as i32;

        // this always works out to be non-negative: when offset_in_bucket is negative for bucket
        // 0, bucket_base_index is still at sub bucket half count, so the sum is positive.
        (bucket_base_signed + offset_in_bucket) as usize
    }
}

