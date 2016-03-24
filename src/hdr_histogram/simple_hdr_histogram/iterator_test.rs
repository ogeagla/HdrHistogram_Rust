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

fn histo64(lowest_discernible_value: u64, highest_trackable_value: u64, num_significant_digits: u32) -> SimpleHdrHistogram<u64> {
    SimpleHdrHistogram::<u64>::new(lowest_discernible_value, highest_trackable_value, num_significant_digits)
}
