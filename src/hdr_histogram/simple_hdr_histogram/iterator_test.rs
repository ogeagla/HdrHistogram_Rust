use std::collections::HashMap;

use hdr_histogram::simple_hdr_histogram::*;

#[test]
fn all_values_all_buckets() {
    let mut h = histo64(1, 8191, 3);

    h.record_single_value(1);
    h.record_single_value(2);
    // first in top half
    h.record_single_value(1024);
    // first in 2nd bucket
    h.record_single_value(2048);
    // first in 3rd
    h.record_single_value(4096);
    // smallest value in last sub bucket of third
    h.record_single_value(8192 - 4);

    let mut actual = HashMap::new();

    for v in h.all_values() {
        actual.insert(v.get_value_iterated_to(), v.get_count_at_value_iterated_to());
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

    h.record_single_value(4);
    // first in top half
    h.record_single_value(4096);
    // first in second bucket
    h.record_single_value(8192);
    // smallest value in last sub bucket of second
    h.record_single_value(16384 - 8);

    let mut actual = HashMap::new();

    for v in h.all_values() {
        actual.insert(v.get_value_iterated_to(), v.get_count_at_value_iterated_to());
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

    h.record_single_value(1);
    h.record_single_value(2);
    // first in top half
    h.record_single_value(1024);
    // first in 2nd bucket
    h.record_single_value(2048);
    // first in 3rd
    h.record_single_value(4096);
    // smallest value in last sub bucket of third
    h.record_single_value(8192 - 4);

    let mut counts = Vec::new();
    let mut values = Vec::new();

    for v in h.recorded_values() {
        counts.push(v.get_count_at_value_iterated_to());
        values.push(v.get_value_iterated_to());
    }

    assert_eq!(vec!(1, 1, 1, 1, 1, 1), counts);
    assert_eq!(vec!(1, 2, 1024, 2048 + 1, 4096 + 3, 8192 - 1), values);
}

#[test]
fn recorded_values_all_buckets_unit_magnitude_2() {
    let mut h = histo64(4, 16384 - 1, 3);

    h.record_single_value(4);
    // first in top half
    h.record_single_value(4096);
    // first in second bucket
    h.record_single_value(8192);
    // smallest value in last sub bucket of second
    h.record_single_value(16384 - 8);

    let mut counts = Vec::new();
    let mut values = Vec::new();

    for v in h.recorded_values() {
        counts.push(v.get_count_at_value_iterated_to());
        values.push(v.get_value_iterated_to());
    }

    assert_eq!(vec!(1, 1, 1, 1), counts);
    assert_eq!(vec!(4 + 3, 4096 + 3, 8192 + 7, 16384 - 1), values);
}

#[cfg(test)]
fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}
