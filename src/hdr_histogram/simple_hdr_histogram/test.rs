use hdr_histogram::simple_hdr_histogram::*;

#[test]
fn count_at_value_on_empty() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(0, h.get_count_at_value(1).unwrap());
    assert_eq!(0, h.get_count_at_value(5000).unwrap());
    assert_eq!(0, h.get_count_at_value(100_000).unwrap());
}

#[test]
fn count_at_value_after_record() {
    let mut h = histo64(1, 100_000, 3);

    h.record_single_value(5000).unwrap();

    assert_eq!(0, h.get_count_at_value(1).unwrap());
    assert_eq!(1, h.get_count_at_value(5000).unwrap());
    assert_eq!(0, h.get_count_at_value(100_000).unwrap());
}

#[test]
fn get_count_at_value_value_below_min() {
    let mut h = histo64(1024, 100_000, 3);

    h.record_single_value(1).unwrap();

    assert_eq!(10, h.unit_magnitude);
    assert_eq!(1023, h.unit_magnitude_mask);
    assert_eq!(43, h.leading_zeros_count_base);

    // maps to bucket 0, sub bucket index 0
    assert_eq!(1, h.get_count_at_value(1).unwrap());
    // maps to b 0, sb 0
    assert_eq!(1, h.get_count_at_value(1023).unwrap());
    assert_eq!(0, h.get_count_at_value(1024).unwrap());
    assert_eq!(0, h.get_count_at_value(100_000).unwrap());
}

#[test]
fn get_count_at_value_value_above_max() {
    let mut h = histo64(1, 100_000, 3);

    // top of 6th bucket is 2^6 * 2047
    h.record_single_value(2047 * 64).unwrap();

    assert_eq!(1, h.get_count_at_value(2047 * 64).unwrap());
    // much bigger value is clamped
    assert_eq!(1, h.get_count_at_value(100_000_000_000).unwrap());
}

#[test]
fn get_count_at_value_after_record_into_top_of_top_bucket_past_stated_max() {
    // this will fit inside 2048 * 2^6, so 7 buckets.
    let mut h = histo64(1, 100_000, 3);

    assert_eq!((7 + 1) * 1024, h.counts.len());

    // 100000 - 2^16 = 34464. In units of 64, this is 538.5. Effective start of 7th bucket is
    // (6 + 1) * 1024.
    assert_eq!(538 + (6 + 1) * 1024, h.counts_array_index(100_000));
    // can record at max.
    h.record_single_value(100_000).unwrap();
    assert_eq!(1, h.get_count_at_value(100_000).unwrap());
    assert_eq!(100000, h.get_max());

    // however, can also record up to 2^17 - 1.
    let max_expressible_val = (2 << 16) - 1;
    assert_eq!(0, h.get_count_at_value(max_expressible_val).unwrap());
    // giant values get clamped to this
    assert_eq!(0, h.get_count_at_value(100_000_000_000).unwrap());

    h.record_single_value(max_expressible_val).unwrap();

    assert_eq!(1, h.get_count_at_value(max_expressible_val).unwrap());
    assert_eq!(1, h.get_count_at_value(100_000_000_000).unwrap());

    // and at stated max is still 1
    assert_eq!(1, h.get_count_at_value(100_000).unwrap());

    assert_eq!(max_expressible_val, h.get_max());

}

#[test]
fn get_count_after_record() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(1, h.get_count());
}

#[test]
fn get_count_after_record_twice() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(5000).unwrap();
    h.record_single_value(5001).unwrap();

    assert_eq!(2, h.get_count());
}

#[test]
fn get_count_empty() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(1, h.get_count());
}

#[test]
fn get_min_non_zero_empty() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(u64::max_value(), h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record_0() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(0).unwrap();

    assert_eq!(u64::max_value(), h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(3).unwrap();

    assert_eq!(3, h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record_below_unit_magnitude_2() {
    let mut h = histo64(4, 100_000, 3);
    h.record_single_value(3).unwrap();

    assert_eq!(u64::max_value(), h.get_min_non_zero());
}


#[test]
fn get_min_non_zero_after_record_at_unit_magnitude_2() {
    let mut h = histo64(4, 100_000, 3);
    h.record_single_value(4).unwrap();

    assert_eq!(4, h.get_min_non_zero());
}

#[test]
fn get_min_non_zero_after_record_above_unit_magnitude_2() {
    let mut h = histo64(4, 100_000, 3);
    h.record_single_value(5).unwrap();

    assert_eq!(4, h.get_min_non_zero());
}

#[test]
fn get_max_after_record() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(5000, h.get_max());
}

#[test]
fn get_max_after_record_unit_magnitude_2() {
    let mut h = histo64(4, 100_000, 3);
    h.record_single_value(5000).unwrap();

    assert_eq!(3, h.unit_magnitude_mask);
    assert_eq!(5003, h.get_max());
}

#[test]
fn get_max_record_smaller_value_doesnt_update_max() {
    let mut h = histo64(1, 100_000, 3);
    h.record_single_value(5000).unwrap();
    h.record_single_value(2000).unwrap();

    assert_eq!(5000, h.get_max());
}

#[test]
fn get_max_empty() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(0, h.get_max());
}

#[test]
fn highest_equivalent_value_unit_magnitude_0() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(0, h.highest_equivalent_value(0));
    assert_eq!(1, h.highest_equivalent_value(1));
    assert_eq!(1023, h.highest_equivalent_value(1023));
    // first in top half
    assert_eq!(1024, h.highest_equivalent_value(1024));
    // last in top half
    assert_eq!(2047, h.highest_equivalent_value(2047));
    // first in 2nd bucket
    assert_eq!(2049, h.highest_equivalent_value(2048));
    assert_eq!(2049, h.highest_equivalent_value(2049));
    // end of 2nd bucket
    assert_eq!(4095, h.highest_equivalent_value(4095));
}

#[test]
fn highest_equivalent_value_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(3, h.highest_equivalent_value(0));
    assert_eq!(3, h.highest_equivalent_value(1));
    assert_eq!(3, h.highest_equivalent_value(3));
    assert_eq!(7, h.highest_equivalent_value(4));
    assert_eq!(4095, h.highest_equivalent_value(4095));
    // first in top half
    assert_eq!(4099, h.highest_equivalent_value(4096));
    // last in top half
    assert_eq!(8191, h.highest_equivalent_value(8188));
    // first in 2nd bucket
    assert_eq!(8192 + 7, h.highest_equivalent_value(8192));
    // 2nd bucket has a scale of 8
    assert_eq!(8192 + 7, h.highest_equivalent_value(8192 + 7));
    // end of 2nd bucket
    assert_eq!(16384 - 1, h.highest_equivalent_value(16384 - 7));
}

#[test]
fn next_non_equivalent_value_unit_magnitude_0() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(1, h.next_non_equivalent_value(0));
    assert_eq!(2, h.next_non_equivalent_value(1));
    assert_eq!(1024, h.next_non_equivalent_value(1023));
    // first in top half
    assert_eq!(1025, h.next_non_equivalent_value(1024));
    // last in top half
    assert_eq!(2048, h.next_non_equivalent_value(2047));
    // first in 2nd bucket
    assert_eq!(2050, h.next_non_equivalent_value(2048));
    // but 2nd bucket has a scale of 2, so next value is same
    assert_eq!(2050, h.next_non_equivalent_value(2049));
    // end of 2nd bucket
    assert_eq!(4096, h.next_non_equivalent_value(4095));
}

#[test]
fn next_non_equivalent_value_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(4, h.next_non_equivalent_value(0));
    assert_eq!(4, h.next_non_equivalent_value(1));
    assert_eq!(4, h.next_non_equivalent_value(3));
    assert_eq!(8, h.next_non_equivalent_value(4));
    assert_eq!(4096, h.next_non_equivalent_value(4095));
    // first in top half
    assert_eq!(4100, h.next_non_equivalent_value(4096));
    // last in top half
    assert_eq!(8192, h.next_non_equivalent_value(8188));
    // first in 2nd bucket
    assert_eq!(8192 + 8, h.next_non_equivalent_value(8192));
    // 2nd bucket has a scale of 8
    assert_eq!(8192 + 8, h.next_non_equivalent_value(8192 + 7));
    // end of 2nd bucket
    assert_eq!(16384, h.next_non_equivalent_value(16384 - 7));
}

#[test]
fn lowest_equivalent_value_unit_magnitude_0() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(0, h.lowest_equivalent_value(0));
    assert_eq!(1, h.lowest_equivalent_value(1));
    assert_eq!(1023, h.lowest_equivalent_value(1023));
    // first in top half
    assert_eq!(1024, h.lowest_equivalent_value(1024));
    // last in top half
    assert_eq!(2047, h.lowest_equivalent_value(2047));
    // first in 2nd bucket
    assert_eq!(2048, h.lowest_equivalent_value(2048));
    // but 2nd bucket has a scale of 2, so next value is same
    assert_eq!(2048, h.lowest_equivalent_value(2049));
    // end of 2nd bucket
    assert_eq!(4094, h.lowest_equivalent_value(4095));
}

#[test]
fn lowest_equivalent_value_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(0, h.lowest_equivalent_value(0));
    assert_eq!(0, h.lowest_equivalent_value(1));
    assert_eq!(0, h.lowest_equivalent_value(3));
    assert_eq!(4, h.lowest_equivalent_value(4));
    // last in bottom half
    assert_eq!(1024 * 4 - 4, h.lowest_equivalent_value(1024 * 4 - 1));
    // first in top half
    assert_eq!(1024 * 4, h.lowest_equivalent_value(1024 * 4));
    // last in top half
    assert_eq!(2048 * 4 - 4, h.lowest_equivalent_value(2048 * 4 - 1));
    // first in 2nd bucket
    assert_eq!(8192, h.lowest_equivalent_value(8192));
    // 2nd bucket has a scale of 8
    assert_eq!(8192, h.lowest_equivalent_value(8192 + 7));
    // end of 2nd bucket
    assert_eq!(16384 - 8, h.lowest_equivalent_value(16384 - 1));
}

#[test]
fn value_from_index_sub_unit_magnitude_0() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(0, h.value_from_index_sub(0, 0));
    assert_eq!(2048 - 1, h.value_from_index_sub(0, 2047));
    assert_eq!(2048, h.value_from_index_sub(1, 1024));
    // scale is 2
    assert_eq!(4096 - 2, h.value_from_index_sub(1, 2047));
    assert_eq!(4096, h.value_from_index_sub(2, 1024));
}

#[test]
fn value_from_index_sub_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(0, h.value_from_index_sub(0, 0));
    assert_eq!(2048 * 4 - 4, h.value_from_index_sub(0, 2047));
    assert_eq!(2048 * 4, h.value_from_index_sub(1, 1024));
    // scale is 8
    assert_eq!(4096 * 4 - 8, h.value_from_index_sub(1, 2047));
    assert_eq!(4096 * 4, h.value_from_index_sub(2, 1024));
}

#[test]
fn get_bucket_index_smallest_value_in_first_bucket() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(0, h.get_bucket_index(0))
}

#[test]
fn get_bucket_index_biggest_value_in_first_bucket() {
    let h = histo64(1, 100_000, 3);
    // sub bucket size 2048, and first bucket uses all 2048 slots
    assert_eq!(0, h.get_bucket_index(2047))
}

#[test]
fn get_bucket_index_smallest_value_in_second_bucket() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(1, h.get_bucket_index(2048))
}

#[test]
fn get_bucket_index_biggest_value_in_second_bucket() {
    let h = histo64(1, 100_000, 3);
    // second value uses only 1024 slots, but scales by 2
    assert_eq!(1, h.get_bucket_index(4095))
}

#[test]
fn get_bucket_index_smallest_value_in_third_bucket() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(2, h.get_bucket_index(4096))
}

#[test]
fn get_bucket_index_smallest_value_in_last_bucket() {
    let h = histo64(1, 100_000, 3);

    // 7 buckets total
    assert_eq!(6, h.get_bucket_index(65536))
}

#[test]
fn get_bucket_index_value_below_smallest_clamps_to_zero() {
    let h = histo64(1024, 100_000, 3);

    // masking clamps bucket index to 0
    assert_eq!(0, h.get_bucket_index(0));
    assert_eq!(0, h.get_bucket_index(1));
    assert_eq!(0, h.get_bucket_index(1023));
    assert_eq!(0, h.get_bucket_index(1024))
}

#[test]
fn get_bucket_index_value_above_biggest_isnt_clamped_at_max_bucket() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(6, h.get_bucket_index(100_000));
    // not clamped; it's just got fewer leading zeros...
    // 2048 * 2^26 = 137,438,953,472
    assert_eq!(26, h.get_bucket_index(100_000_000_000));
}

#[test]
fn get_sub_bucket_index_zero_value_in_first_bucket() {
    let h = histo64(1, 100_000, 3);
    // below min distinguishable value, but still gets bucketed into 0
    let value = 0;
    assert_eq!(0, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_distinguishable_value_in_first_bucket() {
    let h = histo64(1, 100_000, 3);
    let value = 1;
    assert_eq!(1, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_zero_value_in_first_bucket_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);
    let value = 0;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(0, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smaller_than_distinguishable_value_in_first_bucket_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);
    let value = 3;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(0, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_distinguishable_value_in_first_bucket_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);
    let value = 4;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(1, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_largest_value_in_first_bucket_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);
    let value = 2048 * 4 - 1;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(2047, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_value_in_second_bucket_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);
    let value = 2048 * 4;
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(1024, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_largest_value_in_first_bucket() {
    let h = histo64(1, 100_000, 3);
    let value = 2047;
    assert_eq!(2047, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_value_in_second_bucket() {
    let h = histo64(1, 100_000, 3);
    let value = 2048;

    // at midpoint of bucket, which is the first position actually used in second bucket
    assert_eq!(1024, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_biggest_value_in_second_bucket() {
    let h = histo64(1, 100_000, 3);
    let value = 4095;

    // at endpoint of bucket, which is the last position actually used in second bucket
    assert_eq!(2047, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_smallest_value_in_third_bucket() {
    let h = histo64(1, 100_000, 3);
    let value = 4096;

    assert_eq!(1024, h.get_sub_bucket_index(value, h.get_bucket_index(value)))
}

#[test]
fn get_sub_bucket_index_value_below_smallest_clamps_to_zero() {
    let h = histo64(1024, 100_000, 3);

    // masking clamps bucket index to 0
    assert_eq!(0, h.get_sub_bucket_index(0, 0));
    assert_eq!(0, h.get_sub_bucket_index(1, 0));
    assert_eq!(0, h.get_sub_bucket_index(1023, 0));
    assert_eq!(1, h.get_sub_bucket_index(1024, 0))
}

#[test]
fn get_sub_bucket_index_value_above_biggest_doesnt_freak_out() {
    let h = histo64(1, 1024 * 1024, 3);

    // normal case:
    // in bucket index 6, scales by 2^6 = 64, start is at 65536.
    // 100_000 - 65536 = 34_464. 34464 / 64 = 538.5. +1024 = 1562
    assert_eq!(1562, h.get_sub_bucket_index(100_000, h.get_bucket_index(100_000)));

    // still in sub bucket count but nonsensical
    // In bucket 26, effective start is 1024 * 2^26 = 68,719,476,736.
    // 100b - start = 31,280,523,264. That / 2^26 = 466.1.
    assert_eq!(466 + 1024, h.get_sub_bucket_index(100_000_000_000, h.get_bucket_index(100_000_000_000)));
}


#[test]
fn counts_array_index_sub_first_bucket_first_entry() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(0, h.counts_array_index_sub(0, 0));
}

#[test]
fn counts_array_index_sub_first_bucket_first_distinguishable_entry() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(1, h.counts_array_index_sub(0, 1));
}

#[test]
fn counts_array_index_sub_first_bucket_last_entry() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(2047, h.counts_array_index_sub(0, 2047));
}

#[test]
fn counts_array_index_sub_second_bucket_first_entry() {
    let h = histo64(1, 100_000, 3);
    // halfway thru bucket, but bottom half is ignored on non-first bucket, so ends up at end of
    // first bucket + 1
    assert_eq!(2048, h.counts_array_index_sub(1, 1024));
}

#[test]
fn counts_array_index_sub_second_bucket_last_entry() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(2048 + 1023, h.counts_array_index_sub(1, 2047));
}

#[test]
fn counts_array_index_second_bucket_last_entry() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(2048 + 1023, h.counts_array_index(4095));
}

#[test]
fn counts_array_index_second_bucket_first_entry() {
    let h = histo64(1, 100_000, 3);
    assert_eq!(2048, h.counts_array_index(2048));
}

#[test]
fn counts_array_index_below_smallest() {
    let h = histo64(1024, 100_000, 3);

    assert_eq!(0, h.counts_array_index(512));
}

#[test]
fn counts_array_index_way_past_largest_value_exceeds_length() {
    let h = histo64(1, 100_000, 3);

    // 7 * 1024 + 1 more 1024
    assert_eq!(8 * 1024, h.counts.len());

    // 2^39 = 1024 * 2^29, so this should be the start of the 30th bucket.
    // Start index is (bucket index + 1) * 1024.
    assert_eq!(1024 * (30 + 1), h.counts_array_index(1 << 40));
}

#[test]
fn normalize_index_zero_offset_doesnt_change_index() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(1234, h.normalize_index(1234, 0, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_simple() {
    let h = histo64(1, 100_000, 3);
    let i = h.counts_array_index(4096);

    // end of second bucket
    assert_eq!(3072, i);
    // equivalent of one right shift. This will not lead to either over or underflow in index
    // normalization.
    assert_eq!(2048, h.normalize_index(i, 1024, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_negative_intermediate() {
    let h = histo64(1, 5000, 3);
    let i = h.counts_array_index(1500);

    // upper half of first bucket
    assert_eq!(1500, i);
    // equivalent of two right shift. This goes negative and has length added back in.
    assert_eq!(1500 - 2048 + 4096,
        h.normalize_index(i, 2048, h.counts.len()).unwrap() as i64)
}

#[test]
fn normalize_index_oversized_intermediate() {
    let h = histo64(1, 5000, 3);
    let i = h.counts_array_index(4096);

    // upper half of first bucket
    assert_eq!(3072, i);
    // equivalent of two left shift. This exceeds array length and has length subtracted.
    assert_eq!(3072 + 2048 - 4096,
        h.normalize_index(i, 2048, h.counts.len()).unwrap() as i64)
}

#[test]
fn init_sub_bucket_count_medium_precision() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(2048_usize, h.sub_bucket_count);
    assert_eq!(1024_usize, h.sub_bucket_half_count);
    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2047, h.sub_bucket_mask);
}

#[test]
fn init_sub_bucket_count_min_precision() {
    let h = histo64(1, 100_000, 0);

    assert_eq!(2_usize, h.sub_bucket_count);
    assert_eq!(1_usize, h.sub_bucket_half_count);
    assert_eq!(0, h.sub_bucket_half_count_magnitude);
    assert_eq!(1, h.sub_bucket_mask);
}

#[test]
fn init_sub_bucket_count_max_precision() {
    let h = histo64(1, 100_000, 5);

    assert_eq!(262144_usize, h.sub_bucket_count);
    assert_eq!(131072usize, h.sub_bucket_half_count);
    assert_eq!(17, h.sub_bucket_half_count_magnitude);
    assert_eq!(262143, h.sub_bucket_mask);
}

#[test]
fn init_sub_bucket_count_medium_precision_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(2048_usize, h.sub_bucket_count);
    assert_eq!(1024_usize, h.sub_bucket_half_count);
    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2047 << 2, h.sub_bucket_mask);
}

#[test]
fn init_unit_magnitude_mask_1() {
    let h = histo64(1, 100_000, 0);

    assert_eq!(0, h.unit_magnitude);
    assert_eq!(0, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_2() {
    let h = histo64(2, 100_000, 0);

    assert_eq!(1, h.unit_magnitude);
    assert_eq!(1, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_3() {
    let h = histo64(3, 100_000, 0);

    assert_eq!(1, h.unit_magnitude);
    assert_eq!(1, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_4() {
    let h = histo64(4, 100_000, 0);

    assert_eq!(2, h.unit_magnitude);
    assert_eq!(3, h.unit_magnitude_mask);
}

#[test]
fn init_unit_magnitude_mask_1000() {
    let h = histo64(1000, 100_000, 0);

    assert_eq!(9, h.unit_magnitude);
    assert_eq!(511, h.unit_magnitude_mask);
}

#[test]
fn buckets_needed_for_value_small() {
    assert_eq!(1, SimpleHdrHistogram::<u64>::buckets_needed_for_value(1900, 2048_usize, 0));
}

#[test]
fn buckets_needed_for_value_med() {
    // 2048 * 2^6 > 100_000, so 7 buckets total
    assert_eq!(7, SimpleHdrHistogram::<u64>::buckets_needed_for_value(100_000, 2048_usize, 0));
}

#[test]
fn buckets_needed_for_value_med_unit_magnitude_2() {
    // (2048 << 2) * 2^4 > 100_000, so 5 buckets total
    assert_eq!(5, SimpleHdrHistogram::<u64>::buckets_needed_for_value(100_000, 2048_usize, 2));
}

#[test]
fn buckets_needed_for_value_big() {
    // should hit the case where it detects impending overflow
    // 2^53 * 2048 == 2^64, so that's 54 buckets (2^0 to 2^53)
    assert_eq!(54, SimpleHdrHistogram::<u64>::buckets_needed_for_value((1_u64 << 63), 2048_usize, 0));
}

#[test]
fn init_count_array_len_1_bucket() {
    let h = histo64(1, 1000, 3);
    // sub_bucket_count = 2048, and that > max value
    assert_eq!(2048, h.counts.len())
}

#[test]
fn init_count_array_len_2_bucket() {
    let h = histo64(1, 4000, 3);
    // sub_bucket_count = 2048, 1 more bucket-worth of sub_bucket_half_count to reach 4096
    assert_eq!(3072, h.counts.len())
}

#[test]
fn init_count_array_len_3_bucket() {
    let h = histo64(1, 5000, 3);
    // 0-2047, 2048-4095 by 2, 4096-8191 by 4
    assert_eq!(4096, h.counts.len())
}

#[test]
fn init_leading_zero_count_base() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(0, h.unit_magnitude);
    assert_eq!(53_usize, h.leading_zeros_count_base)
}

#[test]
fn init_leading_zero_count_base_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(10, h.sub_bucket_half_count_magnitude);
    assert_eq!(2, h.unit_magnitude);
    assert_eq!(51_usize, h.leading_zeros_count_base)
}

fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}