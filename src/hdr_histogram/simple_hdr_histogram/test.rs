use hdr_histogram::simple_hdr_histogram::*;

#[test]
fn count_at_value_on_empty() {
    let h = init_histo(1, 100000, 3);

    assert_eq!(h.get_count_at_value(1).unwrap(), 0);
    assert_eq!(h.get_count_at_value(5000).unwrap(), 0);
    assert_eq!(h.get_count_at_value(100000).unwrap(), 0);
}

#[test]
fn count_at_value_after_record() {
    let mut h = init_histo(1, 100000, 3);

    h.record_single_value(5000).unwrap();

    assert_eq!(h.get_count_at_value(1).unwrap(), 0);
    assert_eq!(h.get_count_at_value(5000).unwrap(), 1);
    assert_eq!(h.get_count_at_value(100000).unwrap(), 0);
}

#[test]
fn can_get_count_after_record() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();

    let count = h.get_count();
    assert_eq!(count, 1);
}

#[test]
fn can_get_max_after_record() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();

    let max = h.get_max();
    assert_eq!(max, 5000);
}

#[test]
fn can_record_single_value() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();
}

#[test]
fn can_compute_indexes_for_smallest_value() {
    let h = init_histo(1, 100000, 3);
    let value = 1;
    assert_eq!(h.get_bucket_index(value), 0);
    assert_eq!(h.get_sub_bucket_index(value, 0), 1);
    assert_eq!(h.counts_array_index(value), 1);
}

#[test]
fn can_compute_counts_array_index() {
    let h = init_histo(1, 100000, 3);
    let result = h.counts_array_index(5000);

    assert_eq!(result, 3298);
}

#[test]
fn can_get_bucket_index() {
    let h = init_histo(1, 100000, 3);
    let result = h.get_bucket_index(5000);
    assert_eq!(result, 2)
}

#[test]
fn can_get_sub_bucket_index() {
    let h = init_histo(1, 100000, 3);
    let result = h.get_sub_bucket_index(5000, 2);
    assert_eq!(result, 1250)
}

#[test]
fn sub_bucket_count_medium_precision() {
    let h = init_histo(1, 100000, 3);

    assert_eq!(2048_usize, h.sub_bucket_count);
    assert_eq!(1024_usize, h.sub_bucket_half_count);
    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2047, h.sub_bucket_mask);
}

#[test]
fn sub_bucket_count_min_precision() {
    let h = init_histo(1, 100000, 0);

    assert_eq!(2_usize, h.sub_bucket_count);
    assert_eq!(1_usize, h.sub_bucket_half_count);
    assert_eq!(0, h.sub_bucket_half_count_magnitude);
    assert_eq!(1, h.sub_bucket_mask);
}

#[test]
fn sub_bucket_count_max_precision() {
    let h = init_histo(1, 100000, 5);

    assert_eq!(262144_usize, h.sub_bucket_count);
    assert_eq!(131072usize, h.sub_bucket_half_count);
    assert_eq!(17, h.sub_bucket_half_count_magnitude);
    assert_eq!(262143, h.sub_bucket_mask);
}

/// lowest_discernible_value: must be >= 1
/// highest_trackable_value: must be >= 2 * lowest_discernible_value
/// num_significant_digits: must be <= 5
fn init_histo(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {

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
    // TODO is this cast OK?
    let sub_bucket_mask = ((sub_bucket_count - 1) << unit_magnitude) as u64;

    let counts_arr_len = counts_arr_len(highest_trackable_value, sub_bucket_count, unit_magnitude);
    let bucket_count = buckets_needed_for_value(highest_trackable_value, sub_bucket_count, unit_magnitude);

    let leading_zero_count_base: usize = (64_u32 - unit_magnitude - sub_bucket_half_count_magnitude - 1) as usize;

    SimpleHdrHistogram {
        leading_zeros_count_base: leading_zero_count_base,
        unit_magnitude: unit_magnitude,
        sub_bucket_mask: sub_bucket_mask,
        sub_bucket_count: sub_bucket_count,
        sub_bucket_half_count: sub_bucket_half_count,
        sub_bucket_half_count_magnitude: sub_bucket_half_count_magnitude,
        counts: vec![0; counts_arr_len],
        counts_array_length: counts_arr_len,
        normalizing_index_offset: 0_usize, // 0 for normal Histogram ctor in Java impl
        min_non_zero_value: u64::max_value(),
        total_count: 0,
        max_value: 0,
        unit_magnitude_mask: unit_magnitude_mask
    }
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