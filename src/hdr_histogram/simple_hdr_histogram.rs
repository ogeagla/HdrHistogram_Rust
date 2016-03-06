use std::cmp;

#[derive(Debug)]
pub struct HistogramIterationValue {
    pub value_iterated_to: u64,
    pub value_iterated_from: u64,
    pub count_at_value_iterated_to: u64,
    pub count_added_in_this_iteration_step: u64,
    pub total_count_to_this_value: u64,
    pub total_value_to_this_value: u64,
    pub percentile: f64,
    pub percentile_level_iterated_to: f64,
    pub integer_to_double_value_conversion_ratio: f64,
}

impl Default for HistogramIterationValue {
    fn default() -> HistogramIterationValue {
        HistogramIterationValue {
            value_iterated_to: 0,
            value_iterated_from: 0,
            count_at_value_iterated_to: 0,
            count_added_in_this_iteration_step: 0,
            total_count_to_this_value: 0,
            total_value_to_this_value: 0,
            percentile: 0.0,
            percentile_level_iterated_to: 0.0,
            integer_to_double_value_conversion_ratio: 0.0,
        }
    }
}

impl HistogramIterationValue {
    fn reset(mut self) {
        self = HistogramIterationValue { ..Default::default() };
    }
}

#[derive(Debug)]
pub struct BaseHistogramIterator {
    pub histogram: SimpleHdrHistogram,
    pub saved_histogram_total_raw_count: u64,
    pub current_index: usize,
    pub current_value_at_index: u64,
    pub next_value_at_index: u64,
    pub prev_value_iterated_to: u64,
    pub total_count_to_prev_index: u64,
    pub total_count_to_current_index: u64,
    pub total_value_to_current_index: u64,
    pub array_total_count: u64,
    pub count_at_this_value: u64,
    pub fresh_sub_bucket: bool,
    pub current_iteration_value: HistogramIterationValue,
    pub integer_to_double_value_conversion_ratio: u64,
}

impl BaseHistogramIterator {
    fn reset_iterator(&mut self, histogram: SimpleHdrHistogram) {
        //TODO
    }
}

#[derive(Debug)]
pub struct RecordedValuesIterator {
    //TODO
    pub visitedIndex: u32,
}

impl Iterator for RecordedValuesIterator {
    type Item = HistogramIterationValue;
    fn next(&mut self) -> Option<Self::Item> {
        //TODO
        None
    }
}

///
/// This module contains helpers, but should be extracted into the project top-level
/// min and max impls are gross, but they work
///
mod helpers {
    ///
    /// return whichever double is smaller
    ///
    pub fn min_f64(first: f64, second: f64) -> f64 {
        match first < second {
            true => first,
            false => second
        }
    }
    ///
    /// return whichever double is bigger
    ///
    pub fn max_f64(first: f64, second: f64) -> f64 {
        match first > second {
            true => first,
            false => second
        }
    }
    #[test]
    pub fn min_64_works() {
        assert_eq!(min_f64(1.0, 2.0), 1.0);
        assert_eq!(min_f64(2.0, 2.0), 2.0);
        assert_eq!(min_f64(2.0, 1.0), 1.0);
    }
    #[test]
    pub fn max_64_works() {
        assert_eq!(max_f64(1.0, 2.0), 2.0);
        assert_eq!(max_f64(2.0, 2.0), 2.0);
        assert_eq!(max_f64(2.0, 1.0), 2.0);
    }
}
///
/// This struct essentially encapsulates the "instance variables" of the histogram
///
#[derive(Debug)]
pub struct SimpleHdrHistogram {
    pub leading_zeros_count_base: usize,
    pub sub_bucket_mask: u64,
    pub unit_magnitude: u32,
    pub sub_bucket_count: usize,
    pub sub_bucket_half_count: usize,
    pub sub_bucket_half_count_magnitude: u32,
    pub counts: Vec<u64>,
    pub counts_array_length: usize,
    pub normalizing_index_offset: usize,
    pub max_value: u64,
    pub min_non_zero_value: u64,
    pub unit_magnitude_mask: u64,
    pub total_count: u64,
}

///
/// Implementing this trait (Default) for our struct gives us a nice way to
/// initialize an instance using default args instead of having to provide all of them
///
impl Default for SimpleHdrHistogram {
    fn default() -> SimpleHdrHistogram {
        SimpleHdrHistogram {
            leading_zeros_count_base: 0,
            sub_bucket_mask: 0,
            unit_magnitude: 0,
            sub_bucket_count: 0,
            sub_bucket_half_count: 0,
            sub_bucket_half_count_magnitude: 0,
            counts: Vec::new(),
            counts_array_length: 0,
            normalizing_index_offset: 0,
            max_value: 0,
            min_non_zero_value: u64::max_value(),
            unit_magnitude_mask: 0,
            total_count: 0,
        }
    }
}

pub trait HistogramBase {

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
    fn get_count_at_value(&mut self, value: u64) -> Result<u64, String>;
    fn get_value_at_percentile(&mut self, percentile: f64) -> u64;
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
    fn get_count_at_index(&mut self, index: usize) -> Result<u64, String>;

}

impl HistogramBase for SimpleHdrHistogram {

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

    fn get_value_at_percentile(&mut self, percentile: f64) -> u64 {
        let requested_percentile = helpers::min_f64(percentile, 100.0);
        let mut count_at_percentile = (((requested_percentile / 100.0) * self.get_count() as f64) + 0.5) as u64;
        count_at_percentile = cmp::max(count_at_percentile, 1);
        let mut total_to_current_index: u64 = 0;
        for i in 0..self.counts_array_length {
            let count_at_index = self.get_count_at_index(i as usize);
            match count_at_index {
                Ok(the_index) => {
                    total_to_current_index += the_index;
                    if total_to_current_index >= count_at_percentile {
                        let value_at_index = self.value_from_index(i as usize);
                        return if percentile == 0.0 {
                            self.lowest_equivalent_value(value_at_index)
                        } else {
                            self.highest_equivalent_value(value_at_index)
                        }
                    }
                }
                Err(err) => { return 0 }
            }
        }
        0
    }

    fn get_count_at_index(&mut self, index: usize) -> Result<u64, String> {
        let normalized_index =
            self.normalize_index(index, self.normalizing_index_offset, self.counts_array_length);
        match normalized_index {
            Ok(the_index) =>
                Ok(self.counts[the_index]),
            Err(err) =>
                Err(err)
        }
    }

    fn get_count_at_value(&mut self, value: u64) -> Result<u64, String> {
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
                self.counts[the_index] += 1;
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

#[test]
fn count_at_value_on_empty() {
    let mut the_hist = init_histo(1, 100000, 3);

    assert_eq!(the_hist.get_count_at_value(1).unwrap(), 0);
    assert_eq!(the_hist.get_count_at_value(5000).unwrap(), 0);
    assert_eq!(the_hist.get_count_at_value(100000).unwrap(), 0);
}

#[test]
fn count_at_value_after_record() {
    let mut the_hist = init_histo(1, 100000, 3);

    let result = the_hist.record_single_value(5000);
    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }

    assert_eq!(the_hist.get_count_at_value(1).unwrap(), 0);
    assert_eq!(the_hist.get_count_at_value(5000).unwrap(), 1);
    assert_eq!(the_hist.get_count_at_value(100000).unwrap(), 0);
}

#[test]
fn can_get_count_after_record() {
    let mut the_hist = init_histo(1, 100000, 3);
    let result = the_hist.record_single_value(5000);
    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }

    let count = the_hist.get_count();
    assert_eq!(count, 1);
}

#[test]
fn can_get_max_after_record() {
    let mut the_hist = init_histo(1, 100000, 3);
    let result = the_hist.record_single_value(5000);
    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }

    let max = the_hist.get_max();
    assert_eq!(max, 5000);
}

#[test]
fn can_record_single_value() {
    let mut the_hist = init_histo(1, 100000, 3);
    let result = the_hist.record_single_value(5000);

    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }
}

#[test]
fn can_compute_indexes_for_smallest_value() {
    let the_hist = init_histo(1, 100000, 3);
    let value = 1;
    assert_eq!(the_hist.get_bucket_index(value), 0);
    assert_eq!(the_hist.get_sub_bucket_index(value, 0), 1);
    assert_eq!(the_hist.counts_array_index(value), 1);
}

#[test]
fn can_compute_counts_array_index() {
    let the_hist = init_histo(1, 100000, 3);
    let result = the_hist.counts_array_index(5000);

    assert_eq!(result, 3298);
}

#[test]
fn can_get_bucket_index() {
    let the_hist = init_histo(1, 100000, 3);
    let result = the_hist.get_bucket_index(5000);
    assert_eq!(result, 2)
}

#[test]
fn can_get_sub_bucket_index() {
    let the_hist = init_histo(1, 100000, 3);
    let result = the_hist.get_sub_bucket_index(5000, 2);
    assert_eq!(result, 1250)
}

/// lowest_discernible_value: must be >= 1
/// highest_trackable_value: must be >= 2 * lowest_discernible_value
/// num_significant_digits: must be <= 5
fn init_histo(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram {

    assert!(lowest_discernible_value >= 1);
    assert!(highest_trackable_value >= 2 * lowest_discernible_value);
    assert!(num_significant_digits <= 5);

    let mut hist =  SimpleHdrHistogram { ..Default::default() };

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
    // TODO is this cast OK?
    let sub_bucket_mask = ((sub_bucket_count - 1) << unit_magnitude) as u64;

    let counts_arr_len = counts_arr_len(highest_trackable_value, sub_bucket_count, unit_magnitude);
    let bucket_count = buckets_needed_for_value(highest_trackable_value, sub_bucket_count, unit_magnitude);

    let leading_zero_count_base: usize = (64_u32 - unit_magnitude - sub_bucket_half_count_magnitude - 1) as usize;

    hist.leading_zeros_count_base = leading_zero_count_base;
    hist.sub_bucket_mask = sub_bucket_mask;
    hist.unit_magnitude = unit_magnitude;
    hist.sub_bucket_count = sub_bucket_count;
    hist.sub_bucket_half_count = sub_bucket_half_count;
    hist.sub_bucket_half_count_magnitude = sub_bucket_half_count_magnitude;
    hist.counts = vec![0; counts_arr_len];
    hist.counts_array_length = counts_arr_len;
    hist.normalizing_index_offset = 0_usize; // 0 for normal Histogram ctor in Java impl

    hist
}

fn buckets_needed_for_value(value: u64, sub_bucket_count: usize, unit_magnitude: u32) -> usize {

    // TODO is this cast ok?
    let mut smallest_untrackable_value: u64 = (sub_bucket_count << unit_magnitude) as u64;
    let mut buckets_needed: usize = 1;

    while smallest_untrackable_value <= value {
        if smallest_untrackable_value > u64::max_value() / 2 {
            return buckets_needed + 1;
        }

        smallest_untrackable_value = smallest_untrackable_value << 1;
        buckets_needed += 1;
    }

    return buckets_needed;
}

fn counts_arr_len_for_buckets(buckets: usize, sub_bucket_count: usize) -> usize {
    (buckets + 1) * (sub_bucket_count / 2)
}

fn counts_arr_len(value: u64, sub_bucket_count: usize, unit_magnitude: u32) -> usize {
    counts_arr_len_for_buckets(buckets_needed_for_value(value, sub_bucket_count, unit_magnitude), sub_bucket_count)
}