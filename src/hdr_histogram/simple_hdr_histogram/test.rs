use hdr_histogram::simple_hdr_histogram::*;

#[test]
fn count_at_value_on_empty() {
    let h = init_histo(1, 100000, 3);

    assert_eq!(0, h.get_count_at_value(1).unwrap());
    assert_eq!(0, h.get_count_at_value(5000).unwrap());
    assert_eq!(0, h.get_count_at_value(100000).unwrap());
}

#[test]
fn count_at_value_after_record() {
    let mut h = init_histo(1, 100000, 3);

    h.record_single_value(5000).unwrap();

    assert_eq!(0, h.get_count_at_value(1).unwrap());
    assert_eq!(1, h.get_count_at_value(5000).unwrap());
    assert_eq!(0, h.get_count_at_value(100000).unwrap());
}

#[test]
fn get_count_after_record() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(1, h.get_count());
}

#[test]
fn get_count_after_record_twice() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();
    h.record_single_value(5001).unwrap();

    assert_eq!(2, h.get_count());
}

#[test]
fn get_count_empty() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();

    let count = h.get_count();
    assert_eq!(1, count);
}

#[test]
fn get_min_non_zero_empty() {
    let mut h = init_histo(1, 100000, 3);

    assert_eq!(u64::max_value(), h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record_0() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(0).unwrap();

    assert_eq!(u64::max_value(), h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(3).unwrap();

    assert_eq!(3, h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record_below_unit_magnitude_2() {
    let mut h = init_histo(4, 100000, 3);
    h.record_single_value(3).unwrap();

    assert_eq!(u64::max_value(), h.get_min_non_zero());
}


#[test]
fn get_min_non_zero_after_record_at_unit_magnitude_2() {
    let mut h = init_histo(4, 100000, 3);
    h.record_single_value(4).unwrap();

    assert_eq!(4, h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record_above_unit_magnitude_2() {
    let mut h = init_histo(4, 100000, 3);
    h.record_single_value(5).unwrap();

    assert_eq!(4, h.get_min_non_zero());
}

#[test]
fn get_max_after_record() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(5000, h.get_max());
}

#[test]
fn get_max_after_record_unit_magnitude_2() {
    let mut h = init_histo(4, 100000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(3, h.unit_magnitude_mask);
    assert_eq!(5003, h.get_max());
}

#[test]
fn get_max_record_smaller_value_doesnt_update_max() {
    let mut h = init_histo(1, 100000, 3);
    h.record_single_value(5000).unwrap();
    h.record_single_value(2000).unwrap();

    assert_eq!(5000, h.get_max());
}

#[test]
fn get_max_empty() {
    let mut h = init_histo(1, 100000, 3);

    assert_eq!(0, h.get_max());
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
    assert_eq!(0, h.get_bucket_index(value));
    assert_eq!(1, h.get_sub_bucket_index(value, 0));
    assert_eq!(1, h.counts_array_index(value));
}

#[test]
fn can_compute_counts_array_index() {
    let h = init_histo(1, 100000, 3);
    let result = h.counts_array_index(5000);

    assert_eq!(3298, result);
}

#[test]
fn get_bucket_index_smallest_value_in_first_bucket() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(0, h.get_bucket_index(0))
}

#[test]
fn get_bucket_index_biggest_value_in_first_bucket() {
    let h = init_histo(1, 100000, 3);
    // sub bucket size 2048, and first bucket uses all 2048 slots
    assert_eq!(0, h.get_bucket_index(2047))
}

#[test]
fn get_bucket_index_smallest_value_in_second_bucket() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(1, h.get_bucket_index(2048))
}

#[test]
fn get_bucket_index_biggest_value_in_second_bucket() {
    let h = init_histo(1, 100000, 3);
    // second value uses only 1024 slots, but scales by 2
    assert_eq!(1, h.get_bucket_index(4095))
}

#[test]
fn get_bucket_index_smallest_value_in_third_bucket() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(2, h.get_bucket_index(4096))
}

#[test]
fn get_bucket_index_smallest_value_in_last_bucket() {
    let h = init_histo(1, 100000, 3);

    // 7 buckets total
    assert_eq!(6, h.get_bucket_index(65536))
}

#[test]
fn get_sub_bucket_index_zero_value_in_first_bucket() {
    let h = init_histo(1, 100000, 3);
    // below min distinguishable value, but still gets bucketed into 0
    let value = 0;
    assert_eq!(0, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_distinguishable_value_in_first_bucket() {
    let h = init_histo(1, 100000, 3);
    let value = 1;
    assert_eq!(1, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_zero_value_in_first_bucket_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);
    let value = 0;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(0, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smaller_than_distinguishable_value_in_first_bucket_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);
    let value = 3;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(0, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_distinguishable_value_in_first_bucket_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);
    let value = 4;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(1, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_largest_value_in_first_bucket_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);
    let value = 2048 * 4 - 1;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(2047, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_value_in_second_bucket_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);
    let value = 2048 * 4;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(1024, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_largest_value_in_first_bucket() {
    let h = init_histo(1, 100000, 3);
    let value = 2047;
    assert_eq!(2047, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_value_in_second_bucket() {
    let h = init_histo(1, 100000, 3);
    let value = 2048;

    // at midpoint of bucket, which is the first position actually used in second bucket
    assert_eq!(1024, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_biggest_value_in_second_bucket() {
    let h = init_histo(1, 100000, 3);
    let value = 4095;

    // at endpoint of bucket, which is the last position actually used in second bucket
    assert_eq!(2047, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_value_in_third_bucket() {
    let h = init_histo(1, 100000, 3);
    let value = 4096;

    assert_eq!(1024, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn counts_array_index_sub_first_bucket_first_entry() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(0, h.counts_array_index_sub(0, 0));
}

#[test]
fn counts_array_index_sub_first_bucket_first_distinguishable_entry() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(1, h.counts_array_index_sub(0, 1));
}

#[test]
fn counts_array_index_sub_first_bucket_last_entry() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(2047, h.counts_array_index_sub(0, 2047));
}

#[test]
fn counts_array_index_sub_second_bucket_first_entry() {
    let h = init_histo(1, 100000, 3);
    // halfway thru bucket, but bottom half is ignored on non-first bucket, so ends up at end of
    // first bucket + 1
    assert_eq!(2048, h.counts_array_index_sub(1, 1024));
}

#[test]
fn counts_array_index_sub_second_bucket_last_entry() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(2048 + 1023, h.counts_array_index_sub(1, 2047));
}

#[test]
fn counts_array_index_second_bucket_last_entry() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(2048 + 1023, h.counts_array_index(4095));
}

#[test]
fn counts_array_index_second_bucket_first_entry() {
    let h = init_histo(1, 100000, 3);
    assert_eq!(2048, h.counts_array_index(2048));
}

#[test]
fn normalize_index_zero_offset_doesnt_change_index() {
    let h = init_histo(1, 100000, 3);

    assert_eq!(1234, h.normalize_index(1234, 0, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_simple() {
    let h = init_histo(1, 100000, 3);
    let i = h.counts_array_index(4096);

    // end of second bucket
    assert_eq!(3072, i);
    // equivalent of one right shift. This will not lead to either over or underflow in index
    // normalization.
    assert_eq!(2048, h.normalize_index(i, 1024, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_negative_intermediate() {
    let h = init_histo(1, 5000, 3);
    let i = h.counts_array_index(1500);

    // upper half of first bucket
    assert_eq!(1500, i);
    // equivalent of two right shift. This goes negative and has length added back in.
    assert_eq!(1500 - 2048 + 4096,
        h.normalize_index(i, 2048, h.counts.len()).unwrap() as i64)
}

#[test]
fn normalize_index_oversized_intermediate() {
    let h = init_histo(1, 5000, 3);
    let i = h.counts_array_index(4096);

    // upper half of first bucket
    assert_eq!(3072, i);
    // equivalent of two left shift. This exceeds array length and has length subtracted.
    assert_eq!(3072 + 2048 - 4096,
        h.normalize_index(i, 2048, h.counts.len()).unwrap() as i64)
}

#[test]
fn init_sub_bucket_count_medium_precision() {
    let h = init_histo(1, 100000, 3);

    assert_eq!(2048_usize, h.sub_bucket_count);
    assert_eq!(1024_usize, h.sub_bucket_half_count);
    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2047, h.sub_bucket_mask);
}

#[test]
fn init_sub_bucket_count_min_precision() {
    let h = init_histo(1, 100000, 0);

    assert_eq!(2_usize, h.sub_bucket_count);
    assert_eq!(1_usize, h.sub_bucket_half_count);
    assert_eq!(0, h.sub_bucket_half_count_magnitude);
    assert_eq!(1, h.sub_bucket_mask);
}

#[test]
fn init_sub_bucket_count_max_precision() {
    let h = init_histo(1, 100000, 5);

    assert_eq!(262144_usize, h.sub_bucket_count);
    assert_eq!(131072usize, h.sub_bucket_half_count);
    assert_eq!(17, h.sub_bucket_half_count_magnitude);
    assert_eq!(262143, h.sub_bucket_mask);
}

#[test]
fn init_sub_bucket_count_medium_precision_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);

    assert_eq!(2048_usize, h.sub_bucket_count);
    assert_eq!(1024_usize, h.sub_bucket_half_count);
    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2047 << 2, h.sub_bucket_mask);
}

#[test]
fn init_unit_magnitude_mask_1() {
    let h = init_histo(1, 100000, 0);

    assert_eq!(0, h.unit_magnitude);
    assert_eq!(0, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_2() {
    let h = init_histo(2, 100000, 0);

    assert_eq!(1, h.unit_magnitude);
    assert_eq!(1, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_3() {
    let h = init_histo(3, 100000, 0);

    assert_eq!(1, h.unit_magnitude);
    assert_eq!(1, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_4() {
    let h = init_histo(4, 100000, 0);

    assert_eq!(2, h.unit_magnitude);
    assert_eq!(3, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_1000() {
    let h = init_histo(1000, 100000, 0);

    assert_eq!(9, h.unit_magnitude);
    assert_eq!(511, h.unit_magnitude_mask);
}

#[test]
fn buckets_needed_for_value_small() {
    assert_eq!(1, buckets_needed_for_value(1900, 2048_usize, 0));
}

#[test]
fn buckets_needed_for_value_med() {
    // 2048 * 2^6 > 100000, so 7 buckets total
    assert_eq!(7, buckets_needed_for_value(100000, 2048_usize, 0));
}

#[test]
fn buckets_needed_for_value_med_unit_magnitude_2() {
    // (2048 << 2) * 2^4 > 100000, so 5 buckets total
    assert_eq!(5, buckets_needed_for_value(100000, 2048_usize, 2));
}

#[test]
fn buckets_needed_for_value_big() {
    // should hit the case where it detects impending overflow
    // 2^53 * 2048 == 2^64, so that's 54 buckets (2^0 to 2^53)
    assert_eq!(54, buckets_needed_for_value((1_u64 << 63), 2048_usize, 0));
}

#[test]
fn init_count_array_len_1_bucket() {
    let h = init_histo(1, 1000, 3);
    // sub_bucket_count = 2048, and that > max value
    assert_eq!(2048, h.counts.len())
}

#[test]
fn init_count_array_len_2_bucket() {
    let h = init_histo(1, 4000, 3);
    // sub_bucket_count = 2048, 1 more bucket-worth of sub_bucket_half_count to reach 4096
    assert_eq!(3072, h.counts.len())
}

#[test]
fn init_count_array_len_3_bucket() {
    let h = init_histo(1, 5000, 3);
    // 0-2047, 2048-4095 by 2, 4096-8191 by 4
    assert_eq!(4096, h.counts.len())
}

#[test]
fn init_leading_zero_count_base() {
    let h = init_histo(1, 100000, 3);

    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(0, h.unit_magnitude);
    assert_eq!(53_usize, h.leading_zeros_count_base)
}

#[test]
fn init_leading_zero_count_base_unit_magnitude_2() {
    let h = init_histo(4, 100000, 3);

    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(51_usize, h.leading_zeros_count_base)
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
    // this cast should be safe; see discussion in buckets_needed_for_value on similar cast
    let sub_bucket_mask = (sub_bucket_count as u64 - 1) << unit_magnitude;

    let bucket_count = buckets_needed_for_value(highest_trackable_value, sub_bucket_count, unit_magnitude);
    let counts_arr_len = counts_arr_len(bucket_count, sub_bucket_count);

    // this is a small number (0 - 63) so any usize can hold it
    let leading_zero_count_base: usize = (64_u32 - unit_magnitude - sub_bucket_half_count_magnitude - 1) as usize;

    SimpleHdrHistogram {
        leading_zeros_count_base: leading_zero_count_base,
        unit_magnitude: unit_magnitude,
        sub_bucket_mask: sub_bucket_mask,
        sub_bucket_count: sub_bucket_count,
        sub_bucket_half_count: sub_bucket_half_count,
        sub_bucket_half_count_magnitude: sub_bucket_half_count_magnitude,
        counts: vec![0; counts_arr_len],
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