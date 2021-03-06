use std::cmp;
use std::cmp::Ord;
use num::traits::Zero;
use num::traits::One;
use num::traits::ToPrimitive;

use hdr_histogram::simple_hdr_histogram::iterator::*;

mod iterator;
#[cfg(test)] mod iterator_test;
#[cfg(test)] mod test;

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
    /// Number of leading zeros in the largest value that can fit in bucket 0.
    leading_zeros_count_base: usize,
    /// Biggest value that can fit in bucket 0
    sub_bucket_mask: u64,
    unit_magnitude: u32,
    sub_bucket_count: usize,
    // always at least 1
    sub_bucket_half_count: usize,
    sub_bucket_half_count_magnitude: u32,
    counts: Vec<T>,
    // is at most counts.len(), so i32 is plenty since counts scales exponentially
    /// Index offset (used to express left/right shifts of values)
    normalizing_index_offset: i32,
    max_value: u64,
    min_non_zero_value: u64,
    unit_magnitude_mask: u64,
    total_count: u64,
}

pub trait HistogramBase<T: HistogramCount> {
    // TODO error handling improvements
    fn record_single_value(&mut self, value: u64) -> Result<(), String>;

    /// Returns the number of values stored in this histo
    fn get_count(&self) -> u64;
    /// Returns the count at the specified value (as well as other equivalent values)
    fn get_count_at_value(&self, value: u64) -> Result<T, String>;

    /// Returns the max value stored. Undefined if no values have been stored.
    fn get_max(&self) -> u64;

    /// Returns the minimum value stored. Undefined if no values have been stored.
    fn get_min_non_zero(&self) -> u64;

    /// Returns the value k such that 2^k <= lowest discernible value
    fn get_unit_magnitude(&self) -> u32;

    /// If percentile == 0.0, value is less than or equivalent to all other values. If percentile
    /// > 0.0, returns the value that the given percentage of the overall recorded value entries
    /// in the histogram are either smaller than or equivalent to.
    fn get_value_at_percentile(&self, percentile: f64) -> u64;

    /// Returns the lowest value equivalent to the provided value (equivalent meaning will store
    /// counts in the same memory location)
    fn lowest_equivalent_value(&self, value: u64) -> u64;
    /// Returns the highest value equivalent to the provided value (equivalent meaning will store
    /// counts in the same memory location)
    fn highest_equivalent_value(&self, value: u64) -> u64;
    /// Returns the smallest value that is greater than and not equivalent to the provided value
    fn next_non_equivalent_value(&self, value: u64) -> u64;
    /// Returns the number of distinct values that will map to the same count as the provided value
    fn size_of_equivalent_value_range(&self, value: u64) -> u64;

    /// Iterate across all recorded values
    fn recorded_values(&self) -> RecordedValues<T>;
    /// Iterate across all expressible values, recorded or not
    fn all_values(&self) -> AllValues<T>;
    /// Iterate across exponentially increasing buckets, starting at value_units_in_first_bucket
    /// and increasing by log_base each step until recorded values are exhausted.
    fn logarithmic_bucket_values(&self, value_units_in_first_bucket: u64, log_base: u64)
        -> LogarithmicValues<T>;
    /// Iterate across equal-sized buckets until all recorded values are exhausted.
    fn linear_bucket_values(&self, value_units_per_bucket: u64) -> LinearValues<T>;
    /// Iterate across percentiles until all recorded values are exhausted.
    fn percentiles(&self, percentile_ticks_per_half_distance: u32) -> Percentiles<T>;

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

    fn recorded_values(&self) -> RecordedValues<T> {
        RecordedValues {
            histo: self
        }
    }

    fn all_values(&self) -> AllValues<T> {
        AllValues {
            histo: self
        }
    }

    fn logarithmic_bucket_values(&self, value_units_in_first_bucket: u64, log_base: u64)
            -> LogarithmicValues<T> {
        LogarithmicValues {
            histo: self,
            value_units_in_first_bucket: value_units_in_first_bucket,
            log_base: log_base
        }
    }

    fn linear_bucket_values(&self, value_units_per_bucket: u64) -> LinearValues<T> {
        LinearValues {
            histo: self,
            value_units_per_bucket: value_units_per_bucket
        }
    }

    fn percentiles(&self, percentile_ticks_per_half_distance: u32) -> Percentiles<T> {
        Percentiles {
            histo: self,
            percentile_ticks_per_half_distance: percentile_ticks_per_half_distance
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

    /// Calculates the index in the counts array, taking the index offset (representing any
    /// left/right shifts) into account.
    ///
    /// Left shifts have positive offsets. This means to read an existing count, you need to use
    /// a higher value to reach a higher index that then has the positive offset subtracted off.
    /// Analogously, right shifts have negative offsets.
    ///
    /// When a shift takes place, it is checked for over/underflow, but after a shift has happened,
    /// the user could still want to record a value that might have over/underflowed if it had been
    /// there at the time of the shift but is still within the permissible limits of the histogram.
    /// So, if a left shift has happened and the offset is positive, we wrap underflow from the
    /// resulting subtraction to the top of the array. If a right shift has happened, the offset is
    /// negative, so we wrap overflow to the bottom.
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

    /// For values in bucket 0, returns an index anywhere in the first bucket. For other buckets,
    /// the value is always in the top half of the bucket because of how bucket indexes are
    /// calculated.
    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize {
        // safe cast: sub bucket indexes are at most 2 * 10^precision, so can fit in usize.
        // bucket_indexes are even smaller, so can certainly fit in u32.
        (value >> (bucket_index as u32 + self.unit_magnitude)) as usize
    }

    /// Returns the bucket index for the smallest bucket that can hold the value.
    fn get_bucket_index(&self, value: u64) -> usize {
        // Mask maps small values to bucket 0
        let value_orred = value | self.sub_bucket_mask;
        self.leading_zeros_count_base - (value_orred.leading_zeros() as usize)
    }

    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize {
        assert!(sub_bucket_index < self.sub_bucket_count);
        assert!(bucket_index == 0 || (sub_bucket_index >= self.sub_bucket_half_count));

        // First entry in bucket that will actually be used (half-way through). For bucket 0 we
        // can use the whole bucket but we still start indexing at the middle.
        let bucket_base_index = (bucket_index + 1) << self.sub_bucket_half_count_magnitude;

        // offset_in_bucket can be negative by up to sub_bucket_half_count for bucket 0.
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

pub struct RecordedValues<'a, T: HistogramCount + 'a> {
    histo: &'a SimpleHdrHistogram<T>
}

pub struct AllValues<'a, T: HistogramCount + 'a> {
    histo: &'a SimpleHdrHistogram<T>
}

pub struct LogarithmicValues<'a, T: HistogramCount + 'a> {
    histo: &'a SimpleHdrHistogram<T>,
    value_units_in_first_bucket: u64,
    log_base: u64
}

pub struct LinearValues<'a, T: HistogramCount + 'a> {
    histo: &'a SimpleHdrHistogram<T>,
    value_units_per_bucket: u64
}

pub struct Percentiles<'a, T: HistogramCount + 'a> {
    histo: &'a SimpleHdrHistogram<T>,
    percentile_ticks_per_half_distance: u32
}

pub trait IterationStrategy<'a, T: HistogramCount + 'a> : Sized {
    fn increment_iteration_level(&mut self, iter: &BaseHistogramIterator<'a, T, Self>);
    /// return true if we've reached a position that should be emitted to the consumer of the
    /// Iterable
    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool;

    /// return false if iteration is done and we should return None to the consumer of the
    /// Iterator. Analog of Java impl's hasNext().
    fn allow_further_iteration(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        self._default_allow_further_iteration(iter)
    }

    /// default used by several implementations. Helper to allow overrides to access original logic
    fn _default_allow_further_iteration(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        iter.total_count_to_current_index < iter.array_total_count
    }

    /// the value exposed to the consumer of the iterator at a given iteration point
    fn value_iterated_to(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> u64 {
        iter.histogram.highest_equivalent_value(iter.current_value_at_index)
    }

    fn percentile_iterated_to(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> f64 {
        // default to the current percentile
        (100.0 * iter.total_count_to_current_index as f64) / iter.array_total_count as f64
    }

    /// return a value that is only used as a placeholder in the iterator when mutationg functions
    /// in this struct are called. The value returned by dummy is never actually used for anything;
    /// it is simply the equivalent of a null ptr that temporarily is the value of a field.
    fn dummy() -> Self;
}

#[derive(Debug)]
pub struct BaseHistogramIterator<'a, T: HistogramCount + 'a, S: IterationStrategy<'a, T>> {
    histogram: &'a SimpleHdrHistogram<T>,
    strategy: S,
    saved_histogram_total_raw_count: u64,
    current_index: usize,
    current_value_at_index: u64,
    next_value_at_index: u64,
    prev_value_iterated_to: u64,
    total_count_to_prev_index: u64,
    total_count_to_current_index: u64,
    total_value_to_current_index: u64,
    array_total_count: u64,
    count_at_this_value: T,
    fresh_sub_bucket: bool,
    current_iteration_value: HistogramIterationValue<T>,
    integer_to_double_value_conversion_ratio: f64,
    // needs to hold counts array index but also -1
    visited_index: i32,
}
