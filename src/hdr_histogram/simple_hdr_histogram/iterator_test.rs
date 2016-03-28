use std::collections::HashMap;

use hdr_histogram::simple_hdr_histogram::*;

#[test]
fn all_values_all_buckets() {
    let mut h = histo64(1, 8191, 3);

    h.record_single_value(1).unwrap();
    h.record_single_value(2).unwrap();
    // first in top half
    h.record_single_value(1024).unwrap();
    // first in 2nd bucket
    h.record_single_value(2048).unwrap();
    // first in 3rd
    h.record_single_value(4096).unwrap();
    // smallest value in last sub bucket of third
    h.record_single_value(8192 - 4).unwrap();

    let mut actual = HashMap::new();

    for v in h.all_values() {
        actual.insert(v.value_iterated_to, v.count_at_value_iterated_to);
    }

    // 4096 distinct expressible values
    assert_eq!(2048 + 2 * 1024, actual.len());

    // value to expected count
    let mut expected = HashMap::new();
    expected.insert(1, 1);
    expected.insert(2, 1);
    expected.insert(1024, 1);
    expected.insert(2048 + 1, 1);
    expected.insert(4096 + 3, 1);
    expected.insert(8192 - 1, 1);

    // make sure everything we recorded is there
    for (value, count) in &expected {
        let actual_count = actual.get(value).expect(&format!("Nothing for value {}", value));
        assert_eq!(count, actual_count)
    }

    // make sure everything that's there is correct
    for (value, count) in &actual {
        match expected.get(value) {
            None => assert_eq!(&0_u64, count),
            Some(expected_count) => assert_eq!(expected_count, count)
        }
    }
}

#[test]
fn all_values_all_buckets_unit_magnitude_2() {
    let mut h = histo64(4, 16384 - 1, 3);

    h.record_single_value(4).unwrap();
    // first in top half
    h.record_single_value(4096).unwrap();
    // first in second bucket
    h.record_single_value(8192).unwrap();
    // smallest value in last sub bucket of second
    h.record_single_value(16384 - 8).unwrap();

    let mut actual = HashMap::new();

    for v in h.all_values() {
        actual.insert(v.value_iterated_to, v.count_at_value_iterated_to);
    }

    // magnitude 2 means 2nd bucket is scale of 8 = 2 * 2^2
    assert_eq!(2048 + 1024, actual.len());

    // value to expected count
    let mut expected = HashMap::new();
    expected.insert(4 + 3, 1);
    expected.insert(4096 + 3, 1);
    expected.insert(8192 + 7, 1);
    expected.insert(16384 - 1, 1);

    // make sure everything we recorded is there
    for (value, count) in &expected {
        let actual_count = actual.get(value).expect(&format!("Nothing for value {}", value));
        assert_eq!(count, actual_count)
    }

    // make sure everything that's there is correct
    for (value, count) in &actual {
        match expected.get(value) {
            None => assert_eq!(&0_u64, count),
            Some(expected_count) => assert_eq!(expected_count, count)
        }
    }
}

#[test]
fn recorded_values_all_buckets() {
    let mut h = histo64(1, 8191, 3);

    h.record_single_value(1).unwrap();
    h.record_single_value(2).unwrap();
    // first in top half
    h.record_single_value(1024).unwrap();
    // first in 2nd bucket
    h.record_single_value(2048).unwrap();
    // first in 3rd
    h.record_single_value(4096).unwrap();
    // smallest value in last sub bucket of third
    h.record_single_value(8192 - 4).unwrap();

    let mut counts = Vec::new();
    let mut values = Vec::new();

    for v in h.recorded_values() {
        counts.push(v.count_at_value_iterated_to);
        values.push(v.value_iterated_to);
    }

    assert_eq!(vec!(1, 1, 1, 1, 1, 1), counts);
    assert_eq!(vec!(1, 2, 1024, 2048 + 1, 4096 + 3, 8192 - 1), values);
}

#[test]
fn recorded_values_all_buckets_unit_magnitude_2() {
    let mut h = histo64(4, 16384 - 1, 3);

    h.record_single_value(4).unwrap();
    // first in top half
    h.record_single_value(4096).unwrap();
    // first in second bucket
    h.record_single_value(8192).unwrap();
    // smallest value in last sub bucket of second
    h.record_single_value(16384 - 8).unwrap();

    let mut counts = Vec::new();
    let mut values = Vec::new();

    for v in h.recorded_values() {
        counts.push(v.count_at_value_iterated_to);
        values.push(v.value_iterated_to);
    }

    assert_eq!(vec!(1, 1, 1, 1), counts);
    assert_eq!(vec!(4 + 3, 4096 + 3, 8192 + 7, 16384 - 1), values);
}

#[test]
fn logarithmic_bucket_values_min_1_base_2_all_buckets() {
    let h = prepare_histo_for_logarithmic_iterator();

    let mut counts_per_step = Vec::new();
    let mut counts_per_index = Vec::new();
    let mut values = Vec::new();

    for v in h.logarithmic_bucket_values(1, 2) {
        // note not using per-index count
        counts_per_step.push(v.count_added_in_this_iteration_step);
        counts_per_index.push(v.count_at_value_iterated_to);
        values.push(v.value_iterated_to);
    }

    assert_eq!(vec!(0, 1, 1, 0, 0,  3,  0,  0,   0,   0,   0,    3,    1), counts_per_step);
    assert_eq!(vec!(0, 1, 0, 0, 0,  1,  0,  0,   0,   0,   0,    0,    1), counts_per_index);
    assert_eq!(vec!(0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095), values);
}

#[test]
fn logarithmic_bucket_values_min_4_base_2_all_buckets() {
    let h = prepare_histo_for_logarithmic_iterator();

    let mut counts_per_step = Vec::new();
    let mut counts_per_index = Vec::new();
    let mut values = Vec::new();

    for v in h.logarithmic_bucket_values(4, 2) {
        // note not using per-index count
        counts_per_step.push(v.count_added_in_this_iteration_step);
        counts_per_index.push(v.count_at_value_iterated_to);
        values.push(v.value_iterated_to);
    }

    assert_eq!(vec!(2, 0, 0,  3,  0,  0,   0,   0,   0,    3,    1), counts_per_step);
    assert_eq!(vec!(0, 0, 0,  1,  0,  0,   0,   0,   0,    0,    1), counts_per_index);
    assert_eq!(vec!(3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095), values);
}

#[test]
fn logarithmic_bucket_values_min_1_base_2_all_buckets_unit_magnitude_2() {
    // two buckets
    let mut h = histo64(4, 16383, 3);

    h.record_single_value(3).unwrap();
    h.record_single_value(4).unwrap();

    // inside [2^(4 + 2), 2^(5 + 2)
    h.record_single_value(70).unwrap();
    h.record_single_value(80).unwrap();
    h.record_single_value(90).unwrap();

    // in 2nd half
    h.record_single_value(5000).unwrap();
    h.record_single_value(5100).unwrap();
    h.record_single_value(5200).unwrap();

    // in last sub bucket of 2nd bucket
    h.record_single_value(16384 - 1).unwrap();
    let mut counts_per_step = Vec::new();
    let mut counts_per_index = Vec::new();
    let mut values = Vec::new();

    for v in h.logarithmic_bucket_values(1, 2) {
        // note not using per-index count
        counts_per_step.push(v.count_added_in_this_iteration_step);
        counts_per_index.push(v.count_at_value_iterated_to);
        values.push(v.value_iterated_to);
    }

    // 4 maps to 7 in magnitude 2
    assert_eq!(vec!(1, 0, 0, 1, 0,  0,  0,  3,   0,   0,   0,    0,    0,    3,    1), counts_per_step);
    // first 3 iterations are just getting up to 3, which is still the '0' sub bucket.
    // All at the same index, so count_at_value stays at 1 for the first 3
    assert_eq!(vec!(1, 1, 1, 1, 0,  0,  0,  0,   0,   0,   0,    0,    0,    0,    1), counts_per_index);
    assert_eq!(vec!(0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383), values);
}

#[test]
fn logarithmic_bucket_values_min_1_base_10_all_buckets() {
    let h = prepare_histo_for_logarithmic_iterator();

    let mut counts_per_step = Vec::new();
    let mut counts_per_index = Vec::new();
    let mut values = Vec::new();

    for v in h.logarithmic_bucket_values(1, 10) {
        // note not using per-index count
        counts_per_step.push(v.count_added_in_this_iteration_step);
        counts_per_index.push(v.count_at_value_iterated_to);
        values.push(v.value_iterated_to);
    }

    assert_eq!(vec!(0, 2, 3,  0,   4), counts_per_step);
    assert_eq!(vec!(0, 0, 0,  0,   1), counts_per_index);
    assert_eq!(vec!(0, 9, 99, 999, 9999), values);
}

#[cfg(test)]
fn prepare_histo_for_logarithmic_iterator() -> SimpleHdrHistogram<u64> {
    // two buckets
    let mut h = histo64(1, 4095, 3);

    h.record_single_value(1).unwrap();
    h.record_single_value(2).unwrap();

    // inside [2^4, 2^5)
    h.record_single_value(20).unwrap();
    h.record_single_value(25).unwrap();
    h.record_single_value(31).unwrap();

    // in 2nd half
    h.record_single_value(1500).unwrap();
    h.record_single_value(1600).unwrap();
    h.record_single_value(1700).unwrap();

    // in last sub bucket of 2nd bucket
    h.record_single_value(4096 - 1).unwrap();

    h
}

#[cfg(test)]
fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}
