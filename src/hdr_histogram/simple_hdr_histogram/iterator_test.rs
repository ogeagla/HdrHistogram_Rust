use hdr_histogram::simple_hdr_histogram::*;

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
    let mut h = histo64(4, 8191, 3);

    h.record_single_value(4);
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

    assert_eq!(vec!(1, 1, 1, 1, 1), counts);
    assert_eq!(vec!(4 + 3, 1024 + 3, 2048 + 3, 4096 + 3, 8192 - 1), values);
}

fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}
