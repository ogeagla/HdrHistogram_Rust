use hdr_histogram::simple_hdr_histogram::*;

#[test]
fn has_next_fails_when_hist_updated() {
    let h = histo64(1, 5000, 3);
    // TODO
}

#[test]
fn has_next_fails_when_hist_cleared() {
    // TODO
}

#[test]
fn recorded_values_bottom_bucket() {
    let mut h = histo64(1, 5000, 3);

    h.record_single_value(1);
    h.record_single_value(2);
    h.record_single_value(4);
    h.record_single_value(8);

    let mut counts = Vec::new();
    let mut values = Vec::new();

    for v in h.recorded_values() {
        counts.push(v.get_count_at_value_iterated_to());
        values.push(v.get_value_iterated_to());
    }

    assert_eq!(vec!(1, 1, 1, 1), counts);
    assert_eq!(vec!(1, 2, 4, 8), values);
}

fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}
