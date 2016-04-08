use std::io::Cursor;
use rand::{Rng, SeedableRng, XorShiftRng, thread_rng};

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
fn get_value_at_percentile_empty() {
    let h = histo64(1, 100_000, 3);

    for x in 0 .. 100 {
        assert_eq!(0, h.get_value_at_percentile(x as f64));
    }
}

#[test]
fn get_value_at_percentile_populated() {
    let mut h = histo64(1, 10000, 3);

    // bottom of first bucket
    h.record_single_value(1000).unwrap();
    // top of first bucket
    h.record_single_value(2000).unwrap();
    // second bucket
    h.record_single_value(3000).unwrap();
    h.record_single_value(4000).unwrap();
    // third
    h.record_single_value(5001).unwrap();

    // always has lowest recorded value
    assert_eq!(1000, h.get_value_at_percentile(0.0));
    assert_eq!(1000, h.get_value_at_percentile(10.0));
    assert_eq!(1000, h.get_value_at_percentile(20.0));
    // this rounds up to exactly count of 2 when calculating how many values, so we get 2000
    assert_eq!(2000, h.get_value_at_percentile(30.0));
    assert_eq!(2000, h.get_value_at_percentile(40.0));

    // in second bucket; highest equiv value
    assert_eq!(3001, h.get_value_at_percentile(50.0));
    assert_eq!(3001, h.get_value_at_percentile(60.0));

    assert_eq!(4001, h.get_value_at_percentile(70.0));
    assert_eq!(4001, h.get_value_at_percentile(80.0));

    // third bucket
    assert_eq!(5003, h.get_value_at_percentile(90.0));
    assert_eq!(5003, h.get_value_at_percentile(100.0));
}

#[test]
fn get_value_at_percentile_populated_high_scale() {
    let mut h = histo64(1, 1_000_000, 3);

    // 7th bucket
    h.record_single_value(100_000).unwrap();
    // 8th bucket
    h.record_single_value(200_000).unwrap();
    // 9th
    h.record_single_value(300_000).unwrap();
    h.record_single_value(400_000).unwrap();
    h.record_single_value(500_000).unwrap();

    // always has lowest recorded value
    // scale of 2^6 = 64 in 7th bucket
    // lowest equivalent value for 0.0
    assert_eq!(99_968, h.get_value_at_percentile(0.0));
    // highest equivalent value
    assert_eq!(100_031, h.get_value_at_percentile(10.0));
    assert_eq!(100_031, h.get_value_at_percentile(20.0));

    // next bucket
    assert_eq!(200_063, h.get_value_at_percentile(30.0));
    assert_eq!(200_063, h.get_value_at_percentile(40.0));

    // next bucket
    assert_eq!(300_031, h.get_value_at_percentile(50.0));
    assert_eq!(300_031, h.get_value_at_percentile(60.0));

    assert_eq!(400_127, h.get_value_at_percentile(70.0));
    assert_eq!(400_127, h.get_value_at_percentile(80.0));

    assert_eq!(500_223, h.get_value_at_percentile(90.0));
    assert_eq!(500_223, h.get_value_at_percentile(100.0));
}

#[test]
fn get_value_at_percentile_populated_exceed_desired_count_with_one_large_count() {
    let mut h = histo64(1, 10000, 3);

    // bottom of first bucket
    h.record_single_value(1000).unwrap();
    // top of first bucket
    h.record_single_value(2000).unwrap();
    h.record_single_value(2000).unwrap();
    h.record_single_value(2000).unwrap();
    // third
    h.record_single_value(5001).unwrap();

    // we'll have gotten to 4 values instead of the desired ceil(0.3 * 5) = 2
    assert_eq!(2000, h.get_value_at_percentile(30.0));
}

#[test]
fn size_of_equivalent_value_range_unit_magnitude_0() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(1, h.size_of_equivalent_value_range(0));
    assert_eq!(1, h.size_of_equivalent_value_range(1));
    assert_eq!(1, h.size_of_equivalent_value_range(1023));
    // first in top half
    assert_eq!(1, h.size_of_equivalent_value_range(1024));
    // last in top half
    assert_eq!(1, h.size_of_equivalent_value_range(2047));
    // first in 2nd bucket
    assert_eq!(2, h.size_of_equivalent_value_range(2048));
    assert_eq!(2, h.size_of_equivalent_value_range(2049));
    // end of 2nd bucket
    assert_eq!(2, h.size_of_equivalent_value_range(4095));

    // in 7th bucket
    assert_eq!(1 << 6, h.size_of_equivalent_value_range(100_000));
    // max value in top bucket
    assert_eq!(1 << 6, h.size_of_equivalent_value_range((1 << 17) - 1));
    // even bigger
    assert_eq!(1 << 7, h.size_of_equivalent_value_range((1 << 17)));
}

#[test]
fn size_of_equivalent_value_range_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(4, h.size_of_equivalent_value_range(0));
    assert_eq!(4, h.size_of_equivalent_value_range(1));
    assert_eq!(4, h.size_of_equivalent_value_range(3));
    assert_eq!(4, h.size_of_equivalent_value_range(4));
    assert_eq!(4, h.size_of_equivalent_value_range(4095));
    // first in top half
    assert_eq!(4, h.size_of_equivalent_value_range(4096));
    // last in top half
    assert_eq!(4, h.size_of_equivalent_value_range(8188));
    // first in 2nd bucket
    assert_eq!(8, h.size_of_equivalent_value_range(8192));
    // end of 2nd bucket
    assert_eq!(8, h.size_of_equivalent_value_range(16384 - 7));
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
    // end of first bucket
    assert_eq!(2048 - 1, h.value_from_index_sub(0, 2047));
    // start of second bucket
    assert_eq!(2048, h.value_from_index_sub(1, 1024));
    // scale is 2
    assert_eq!(4096 - 2, h.value_from_index_sub(1, 2047));
    assert_eq!(4096, h.value_from_index_sub(2, 1024));
}

#[test]
fn value_from_index_sub_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    assert_eq!(0, h.value_from_index_sub(0, 0));
    // end of first bucket
    assert_eq!(2048 * 4 - 4, h.value_from_index_sub(0, 2047));
    // start of second bucket
    assert_eq!(2048 * 4, h.value_from_index_sub(1, 1024));
    // scale is 8
    assert_eq!(4096 * 4 - 8, h.value_from_index_sub(1, 2047));
    assert_eq!(4096 * 4, h.value_from_index_sub(2, 1024));
}

#[test]
fn value_from_index_unit_magnitude_0() {
    let h = histo64(1, 100_000, 3);

    // first bucket
    assert_eq!(0, h.value_from_index(0));
    assert_eq!(1023, h.value_from_index(1023));
    assert_eq!(1024, h.value_from_index(1024));
    assert_eq!(2047, h.value_from_index(2047));
    // second bucket
    assert_eq!(2048, h.value_from_index(2048));
    assert_eq!(4096 - 2, h.value_from_index(3071));
}

#[test]
fn value_from_index_unit_magnitude_2() {
    let h = histo64(4, 100_000, 3);

    // first bucket
    assert_eq!(0, h.value_from_index(0));
    assert_eq!(4096 - 4, h.value_from_index(1023));
    assert_eq!(4096, h.value_from_index(1024));
    assert_eq!(8192 - 4, h.value_from_index(2047));
    // second bucket
    assert_eq!(8192, h.value_from_index(2048));
    assert_eq!(16384 - 8, h.value_from_index(3071));
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

// TODO can't shift right into bottom half
// TODO can't shift left past end
// TODO plain shift left
// TODO plain shift right
// TODO shift left and right is same
// TODO shift right and left is same
// TODO shift left, then write range that underflows
// TODO shift left, then write range that underflows, then try to shift more
// TODO shift right, then write range that overflows
// TODO shift right, then write range that overflows, then try to shift more


#[test]
fn normalize_index_zero_offset_doesnt_change_index() {
    let h = histo64(1, 100_000, 3);

    assert_eq!(1234, h.normalize_index(1234, 0, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_simple() {
    let h = histo64(1, 10_000, 3);
    let i = h.counts_array_index(4096);

    // end of second bucket
    assert_eq!(3072, i);
    // equivalent of one left shift. This will not lead to either over or underflow in index
    // normalization.
    assert_eq!(2048, h.normalize_index(i, 1024, h.counts.len()).unwrap());
    // get original location if you ask for twice the original
    assert_eq!(i, h.normalize_index(h.counts_array_index(4096 << 1), 1024, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_left_shift_underflow() {
    let h = histo64(1, 10_000, 3);
    let val = 1500;
    let i = h.counts_array_index(val);
    let len = 1024 * 5;
    let offset = 2048;

    // upper half of first bucket
    assert_eq!(1500, i);
    // equivalent of two left shift. This goes negative and has length added back in.
    assert_eq!(1500 - offset + len, h.normalize_index(i, offset, h.counts.len()).unwrap() as i32);
    assert_eq!(i, h.normalize_index(h.counts_array_index(val << 2), offset, h.counts.len()).unwrap())
}

#[test]
fn normalize_index_right_shift_overflow() {
    let h = histo64(1, 10_000, 3);
    let val = 4096;
    let i = h.counts_array_index(val);
    let len = 1024 * 5;
    let offset = -2048;

    // start of third bucket
    assert_eq!(3072, i);
    // equivalent of two right shift. This goes past end and has length subtracted.
    assert_eq!(3072 - offset - len, h.normalize_index(i, offset, h.counts.len()).unwrap() as i32);
    assert_eq!(i, h.normalize_index(h.counts_array_index(val >> 2), offset, h.counts.len()).unwrap())
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
    assert_eq!(54,
    SimpleHdrHistogram::<u64>::buckets_needed_for_value(u64::max_value(), 2048_usize, 0));
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
fn init_count_array_len_most_buckets_possible() {
    let value = u64::max_value();
    let h = histo64(1, value, 0);

    assert_eq!(2, h.sub_bucket_count);
    // 2^63 * 2^1 = 2^16, so 64 buckets.
    // note that this would fit in an i8 even...
    assert_eq!(64, SimpleHdrHistogram::<u64>::buckets_needed_for_value(
    value, h.sub_bucket_count, h.unit_magnitude));

    assert_eq!(64 + 1, h.counts.len());
}

#[test]
fn init_count_array_len_biggest_array_possible() {
    let value = u64::max_value();
    let h = histo64(1, value, 5);

    // 5 sigdigs = 100,000. sub bucket = 200,000. 2^18 = 262,144.
    assert_eq!(2_usize.pow(18), h.sub_bucket_count);
    // 2^46 * 2^18 = 2^16, so 47 buckets.
    assert_eq!(47, SimpleHdrHistogram::<u64>::buckets_needed_for_value(
    value, h.sub_bucket_count, h.unit_magnitude));

    // still fits in i32
    assert_eq!((47 + 1) * 2_usize.pow(17), h.counts.len());
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

#[test]
fn varint_write_3_bit_value() {
    let buf = &mut Cursor::new(Vec::<u8>::new());
    super::varint_write(6, buf);

    let vec = buf.get_ref();
    assert_eq!(1, vec.len());
    assert_eq!(0x6, vec[0]);
}


#[test]
fn varint_write_7_bit_value() {
    let buf = &mut Cursor::new(Vec::<u8>::new());
    super::varint_write(127, buf);

    let vec = buf.get_ref();
    assert_eq!(1, vec.len());
    assert_eq!(0x7F, vec[0]);
}


#[test]
fn varint_write_9_bit_value() {
    let buf = &mut Cursor::new(Vec::<u8>::new());
    super::varint_write(256, buf);

    let vec = buf.get_ref();
    // marker high bit w/ 0's, then 9th bit (2nd bit of 2nd 7-bit group)
    assert_eq!(&vec![0x80, 0x02], buf.get_ref());
}

#[test]
fn varint_write_u64_max() {
    let buf = &mut Cursor::new(Vec::<u8>::new());
    super::varint_write(u64::max_value(), buf);

    assert_eq!(&vec![0xFF; 9], buf.get_ref());
}

#[test]
fn varint_read_u64_max() {
    let input = &mut Cursor::new(vec![0xFF; 9]);
    assert_eq!(u64::max_value(), super::varint_read(input).unwrap());
}

#[test]
fn varint_read_u64_zero() {
    let input = &mut Cursor::new(vec![0x00; 9]);
    assert_eq!(0, super::varint_read(input).unwrap());
}

#[test]
fn varint_write_read_roundtrip_prng() {
    let mut rng = thread_rng();
    let seed: &[u32; 4] = &[rng.gen::<u32>(), rng.gen::<u32>(), rng.gen::<u32>(), rng.gen::<u32>()];
    println!("Seed: {:?}", seed);

    let mut prng: XorShiftRng = SeedableRng::from_seed(*seed);

    for i in 1..1_000_000 {
        let int: u64 = prng.gen();
        let cursor = &mut Cursor::new(Vec::<u8>::new());
        super::varint_write(int, cursor);
        assert_eq!(int, super::varint_read(&mut Cursor::new(cursor.get_ref())).unwrap());
    }
}

#[test]
fn zig_zag_encode_0() {
    assert_eq!(0, super::zig_zag_encode(0));
}

#[test]
fn zig_zag_encode_neg_1() {
    assert_eq!(1, super::zig_zag_encode(-1));
}

#[test]
fn zig_zag_encode_i64_max() {
    assert_eq!(u64::max_value() - 1, super::zig_zag_encode(i64::max_value()));
}

#[test]
fn zig_zag_encode_i64_min() {
    assert_eq!(u64::max_value(), super::zig_zag_encode(i64::min_value()));
}

#[cfg(test)]
fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u8) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}
