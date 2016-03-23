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
    // always at least 1
    sub_bucket_half_count: usize,
    sub_bucket_half_count_magnitude: u32,
    counts: Vec<T>,
    // is at most counts.len(), so i32 is plenty since counts scales exponentially
    normalizing_index_offset: i32,
    max_value: u64,
    min_non_zero_value: u64,
    unit_magnitude_mask: u64,
    total_count: u64,
}

pub trait HistogramBase<T: HistogramCount> {
    fn record_single_value(&mut self, value: u64) -> Result<(), String>;

    fn get_count(&self) -> u64;
    fn get_count_at_value(&self, value: u64) -> Result<T, String>;

    fn get_max(&self) -> u64;
    fn get_min_non_zero(&self) -> u64;

    fn get_unit_magnitude(&self) -> u32;

    /// If percentile == 0.0, value is less than or equivalent to all other values. If percentile
    /// > 0.0, returns the value that the given percentage of the overall recorded value entries
    /// in the histogram are either smaller than or equivalent to.
    fn get_value_at_percentile(&self, percentile: f64) -> u64;

    fn lowest_equivalent_value(&self, value: u64) -> u64;
    fn highest_equivalent_value(&self, value: u64) -> u64;
    fn next_non_equivalent_value(&self, value: u64) -> u64;
    fn size_of_equivalent_value_range(&self, value: u64) -> u64;
}

impl<T: HistogramCount> HistogramBase<T> for SimpleHdrHistogram<T> {

    fn next_non_equivalent_value(&self, value: u64) -> u64 {
        self.lowest_equivalent_value(value) + self.size_of_equivalent_value_range(value)
    }

    fn size_of_equivalent_value_range(&self, value: u64) -> u64 {
        let bucket_index = self.get_bucket_index(value);
        let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
        // TODO when is sub_bucket_index >= sub_bucket_count
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
        self.value_from_index_sub(bucket_index, sub_bucket_index)
    }

    fn get_value_at_percentile(&self, percentile: f64) -> u64 {
        let requested_percentile = percentile.min(100.0);
        let mut count_at_percentile =
            (((requested_percentile / 100.0) * self.get_count() as f64) + 0.5) as u64;
        count_at_percentile = cmp::max(count_at_percentile, 1);
        let mut total_to_current_index: u64 = 0;
        for i in 0..self.counts.len() {
            let count_at_index = self.get_count_at_index(i as usize);
            match count_at_index {
                Ok(count) => {
                    // we only use u8 - u64 types, so this must always work
                    total_to_current_index += count.to_u64().unwrap();
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

    fn get_count_at_value(&self, value: u64) -> Result<T, String> {
        // TODO is it ok to just clamp to max value rathe than saying it's inexpressible?
        let index = cmp::min(cmp::max(0, self.counts_array_index(value)), self.counts.len() - 1);
        self.get_count_at_index(index)
    }

    fn get_max(&self) -> u64 {
        self.max_value
    }

    fn get_min_non_zero(&self) -> u64 {
        self.min_non_zero_value
    }

    fn get_count(&self) -> u64 {
        self.total_count
    }

    fn get_unit_magnitude(&self) -> u32 {
        self.unit_magnitude
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


}

impl<T: HistogramCount> SimpleHdrHistogram<T> {

    /// lowest_discernible_value: must be >= 1
    /// highest_trackable_value: must be >= 2 * lowest_discernible_value
    /// num_significant_digits: must be <= 5
    pub fn new(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<T> {

        assert!(lowest_discernible_value >= 1);
        assert!(highest_trackable_value >= 2 * lowest_discernible_value);
        assert!(num_significant_digits <= 5);

        let largest_value_with_single_unit_resolution = 2_u64 * 10_u64.pow(num_significant_digits);

        let unit_magnitude = ((lowest_discernible_value as f64).ln() / 2_f64.ln()) as u32;
        let unit_magnitude_mask: u64  = (1_u64 << unit_magnitude) - 1;

        // find nearest power of 2 to largest_value_with_single_unit_resolution
        let sub_bucket_count_magnitude: u32 =
        ((largest_value_with_single_unit_resolution as f64).ln() / 2_f64.ln()).ceil() as u32;

        // ugly looking... how should ternaries be done?
        let sub_bucket_half_count_magnitude: u32 = (if sub_bucket_count_magnitude > 1 { sub_bucket_count_magnitude } else { 1 }) - 1;
        let sub_bucket_count: usize = 2_usize.pow(sub_bucket_half_count_magnitude + 1);
        let sub_bucket_half_count: usize = sub_bucket_count / 2;
        // this cast should be safe; see discussion in buckets_needed_for_value on similar cast
        let sub_bucket_mask = (sub_bucket_count as u64 - 1) << unit_magnitude;

        let bucket_count = SimpleHdrHistogram::<T>::buckets_needed_for_value(highest_trackable_value, sub_bucket_count, unit_magnitude);
        let counts_arr_len = SimpleHdrHistogram::<T>::counts_arr_len(bucket_count, sub_bucket_count);

        // this is a small number (0 - 63) so any usize can hold it
        let leading_zero_count_base: usize = (64_u32 - unit_magnitude - sub_bucket_half_count_magnitude - 1) as usize;

        SimpleHdrHistogram {
            leading_zeros_count_base: leading_zero_count_base,
            unit_magnitude: unit_magnitude,
            sub_bucket_mask: sub_bucket_mask,
            sub_bucket_count: sub_bucket_count,
            sub_bucket_half_count: sub_bucket_half_count,
            sub_bucket_half_count_magnitude: sub_bucket_half_count_magnitude,
            counts: vec![T::zero(); counts_arr_len],
            normalizing_index_offset: 0, // 0 for normal Histogram ctor in Java impl
            min_non_zero_value: u64::max_value(),
            total_count: 0,
            max_value: 0,
            unit_magnitude_mask: unit_magnitude_mask
        }
    }

    fn buckets_needed_for_value(value: u64, sub_bucket_count: usize, unit_magnitude: u32) -> usize {

        // sub_bucket_count is 2 * 10^precision, so fairly small and certainly fits in u64.
        // If unit magnitude is too big, this will panic, but not much we can do about it.
        // Pretty unlikely to have a large unit_magnitude (you'd need at least 46 to cause the max
        // sized sub_bucket_count of 2^18 to overflow...)
        let mut smallest_untrackable_value: u64 = (sub_bucket_count as u64) << unit_magnitude;
        let mut buckets_needed = 1_usize;

        while smallest_untrackable_value <= value {
            if smallest_untrackable_value > u64::max_value() / 2 {
                return buckets_needed + 1;
            }

            smallest_untrackable_value = smallest_untrackable_value << 1;
            buckets_needed += 1;
        }

        return buckets_needed;
    }

    fn counts_arr_len(bucket_count: usize, sub_bucket_count: usize) -> usize {
        (bucket_count + 1) * (sub_bucket_count / 2)
    }

    fn value_from_index(&self, index: usize) -> u64 {
        // Dividing by sub bucket half count will yield 1 in top half of first bucket, 2 in
        // 2nd bucket, etc, so subtract 1.
        // bucket index is 64 max. will go negative for values in lower half
        let mut bucket_index: i32 = (index as i32 >> self.sub_bucket_half_count_magnitude) - 1;
        // Msk to lower half, add in half count to always end up in top half.
        // This will move things in lower half of first bucket into the top half.
        let mut sub_bucket_index: usize = (index & (self.sub_bucket_half_count - 1))
            + self.sub_bucket_half_count;
        if bucket_index < 0 {
            // lower half of first bucket case; move sub bucket index back
            sub_bucket_index -= self.sub_bucket_half_count;
            bucket_index = 0;
        }

        self.value_from_index_sub(bucket_index as usize, sub_bucket_index)
    }

    fn get_count_at_index(&self, index: usize) -> Result<T, String> {
        let normalized_index =
        self.normalize_index(index, self.normalizing_index_offset, self.counts.len());
        match normalized_index {
            Ok(the_index) =>
            Ok(self.counts[the_index]),
            Err(err) =>
            Err(err)
        }
    }

    fn value_from_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> u64 {
        // these indexes are all small, so safe to cast
        (sub_bucket_index as u64) << (bucket_index as u32 + self.unit_magnitude)
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

    fn increment_count_at_index(&mut self, index: usize) -> Result<(), String> {
        let normalized_index =
        self.normalize_index(index, self.normalizing_index_offset, self.counts.len());
        match normalized_index {
            Ok(the_index) => {
                // TODO express exceeding the counts size as an error here?
                self.counts[the_index] = self.counts[the_index] + T::one();
                Ok(())
            }
            Err(err) => Err(err)
        }
    }

    fn normalize_index(&self, index: usize, normalizing_index_offset: i32, array_length: usize) ->
    Result<usize, String> {
        match normalizing_index_offset {
            0 => Ok(index),
            _ =>
            if index > array_length {
                Err(String::from("index out of covered range"))
            } else {
                // indices are always pretty small since they scale to values exponentially
                let array_length_i32 : i32 = array_length as i32;
                let mut normalized_index: i32 = index as i32 - normalizing_index_offset;

                // Calculate unsigned remainder.
                // When offset is positive (right shifted), it's checked at the time of the
                // right shift op to not shift anything into the bottom half of the first
                // bucket or below. Therefore, we know that the offset is less than array
                // length. So, we can simply check for sum < 0 and add one array length.
                // When offset is negative (left shifted), it's also checked at left shift time
                // to ensure it won't cause anything to be shifted past the end of the array.
                // Similarly, we know that |offset| < array length, so if sum > array length,
                // we can just subtract 1 array length.
                if normalized_index < 0 {
                    normalized_index += array_length_i32
                } else if normalized_index >= array_length_i32 {
                    normalized_index -= array_length_i32
                }

                Ok(normalized_index as usize)
            }
        }
    }

    fn counts_array_index(&self, value: u64) -> usize {
        let bucket_index = self.get_bucket_index(value);
        let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
        return self.counts_array_index_sub(bucket_index, sub_bucket_index);
    }

    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize {
        // safe cast: sub bucket indexes are at most 2 * 10^precision, so can fit in usize.
        // bucket_indexes are even smaller, so can certainly fit in u32.
        (value >> (bucket_index as u32 + self.unit_magnitude)) as usize
    }

    fn get_bucket_index(&self, value: u64) -> usize {
        let value_orred = value | self.sub_bucket_mask;
        self.leading_zeros_count_base - (value_orred.leading_zeros() as usize)
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