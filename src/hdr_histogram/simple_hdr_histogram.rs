/*

  This struct essentially encapsulates the "instance variables"

*/
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


/*

  Implementing this trait (Default) for our struct gives us a nice way to
  initialize an instance using default args instead of having to provide all of them

 */
impl Default for SimpleHdrHistogram {
    fn default () -> SimpleHdrHistogram {
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
    fn counts_array_index(&self, value: u64) -> Result<usize, String>;
    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize;
    fn get_bucket_index(&self, value: u64) -> usize;
    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize;
    fn update_min_and_max(&mut self, value: u64);
    fn update_max_value(&mut self, value: u64);
    fn update_min_non_zero_value(&mut self, value: u64);
    // end TODO

    fn increment_count_at_index(&mut self, index: usize) -> Result<(), String>;
    fn normalize_index(&self, index: usize, normalizing_index_offset: usize, array_length: usize) ->
        Result<usize, String>;
    fn increment_total_count(&mut self);


}

impl HistogramBase for SimpleHdrHistogram {

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
        let sum = bucket_index + self.unit_magnitude as usize;
        value.rotate_right(sum as u32) as usize
    }

    fn get_bucket_index(&self, value: u64) -> usize {
        let value_orred = value | self.sub_bucket_mask;
        self.leading_zeros_count_base - (value_orred.leading_zeros() as usize)
    }

    fn record_single_value(&mut self, value: u64) -> Result<(), String> {
        match self.counts_array_index(value) {
            Ok(counts_index) => {
                match self.increment_count_at_index(counts_index) {
                    Ok(_) => {
                        self.update_min_and_max(value);
                        self.increment_total_count();
                        Ok(())
                    }
                    Err(err) => {
                        Err(String::from(format!("Could not increment count at index due to: {}", err))) }
                }}
            Err(err) => Err(String::from(format!("Could not get index due to: {}", err)))
        }
    }

    fn counts_array_index(&self, value: u64) -> Result<usize, String> {
        let bucket_index = self.get_bucket_index(value);
        let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
        let result = self.counts_array_index_sub(bucket_index, sub_bucket_index);
        Ok(result)
    }

    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize {
        assert!(sub_bucket_index < self.sub_bucket_count);
        assert!(bucket_index == 0 || (sub_bucket_index >= self.sub_bucket_half_count));

        let bucket_base_index = (bucket_index + 1) << self.sub_bucket_half_count_magnitude;
        let offset_in_bucket = sub_bucket_index - self.sub_bucket_half_count;

        bucket_base_index + offset_in_bucket
    }
}

#[test]
fn can_record_single_value() {
    let mut the_hist = SimpleHdrHistogram { ..Default::default() };
    let result = the_hist.record_single_value(99);

    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }
}

#[test]
fn can_compute_counts_array_index() {
    let the_hist = init_histo(1, 1000, 5);
    let result = the_hist.counts_array_index(99);

    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not compute counts array index because error: {}", err))
    }
}

#[test]
fn can_get_bucket_index() {
    let the_hist = init_histo(1, 1000, 5);
    let result = the_hist.get_bucket_index(99);
    assert_eq!(result, 0)
}

#[test]
fn can_get_sub_bucket_index() {
    let the_hist = init_histo(1, 1000, 5);
    let result = the_hist.get_sub_bucket_index(99, 1);
    assert_eq!(result, 0)
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
    let unit_magnitude_mask: u64  = 1_u64.rotate_left(unit_magnitude) - 1;

    // find nearest power of 2 to largest_value_with_single_unit_resolution
    let sub_bucket_count_magnitude: u32 =
        ((largest_value_with_single_unit_resolution as f64).ln() / 2_f64.ln()).ceil() as u32;

    // ugly looking... how should ternaries be done?
    let sub_bucket_half_count_magnitude: u32 = (if sub_bucket_count_magnitude > 1 { sub_bucket_count_magnitude } else { 1 }) - 1;
    let sub_bucket_count: usize = 2_usize.pow(sub_bucket_half_count_magnitude + 1);
    let sub_bucket_half_count: usize = sub_bucket_count / 2;
    // TODO is this cast OK?
    let sub_bucket_mask = (sub_bucket_count - 1).rotate_left(unit_magnitude) as u64;

    let counts_arr_len = counts_arr_len(highest_trackable_value, sub_bucket_count, unit_magnitude);
    let bucket_count = buckets_needed_for_value(highest_trackable_value, sub_bucket_count, unit_magnitude);

    let leading_zero_count_base: usize = (64_u32 - unit_magnitude - sub_bucket_half_count_magnitude - 1) as usize;

    hist.leading_zeros_count_base = leading_zero_count_base;
    hist.sub_bucket_mask = sub_bucket_mask;
    hist.unit_magnitude = unit_magnitude;
    hist.sub_bucket_count = sub_bucket_count;
    hist.sub_bucket_half_count = sub_bucket_half_count;
    hist.sub_bucket_half_count_magnitude = sub_bucket_half_count_magnitude;
    hist.counts = Vec::with_capacity(counts_arr_len);
    hist.counts_array_length = counts_arr_len;
    hist.normalizing_index_offset = 0_usize; // 0 for normal Histogram ctor in Java impl

    println!("hist: {:?}", hist);

    hist
}

fn buckets_needed_for_value(value: u64, sub_bucket_count: usize, unit_magnitude: u32) -> usize {

    // is this cast ok?
    let mut smallest_untrackable_value: u64 = sub_bucket_count.rotate_left(unit_magnitude) as u64;
    let mut buckets_needed: usize = 1;

    while smallest_untrackable_value <= value {
        if smallest_untrackable_value > u64::max_value() / 2 {
            return buckets_needed + 1;
        }

        smallest_untrackable_value = smallest_untrackable_value.rotate_left(1);
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