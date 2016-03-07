use hdr_histogram::simple_hdr_histogram::*;

#[derive(Debug)]
pub struct HistogramIterationValue {
    value_iterated_to: u64,
    value_iterated_from: u64,
    count_at_value_iterated_to: u64,
    count_added_in_this_iteration_step: u64,
    total_count_to_this_value: u64,
    total_value_to_this_value: u64,
    percentile: f64,
    percentile_level_iterated_to: f64,
    integer_to_double_value_conversion_ratio: f64,
}

impl Default for HistogramIterationValue {
    fn default() -> HistogramIterationValue {
        HistogramIterationValue {
            value_iterated_to: 0,
            value_iterated_from: 0,
            count_at_value_iterated_to: 0,
            count_added_in_this_iteration_step: 0,
            total_count_to_this_value: 0,
            total_value_to_this_value: 0,
            percentile: 0.0,
            percentile_level_iterated_to: 0.0,
            integer_to_double_value_conversion_ratio: 0.0,
        }
    }
}

impl HistogramIterationValue {
    fn reset(mut self) {
        self = HistogramIterationValue { ..Default::default() };
    }
}

#[derive(Debug)]
pub struct BaseHistogramIterator<T: HistogramCount> {
    histogram: SimpleHdrHistogram<T>,
    saved_histogram_total_raw_count: u64,
    current_index: usize,
    current_value_at_index: u64,
    next_value_at_index: u64,
    prev_value_iterated_to: u64,
    total_count_to_prev_index: u64,
    total_count_to_current_index: u64,
    total_value_to_current_index: u64,
    array_total_count: u64,
    count_at_this_value: u64,
    fresh_sub_bucket: bool,
    current_iteration_value: HistogramIterationValue,
    integer_to_double_value_conversion_ratio: u64,
}

impl<T: HistogramCount> BaseHistogramIterator<T> {
    fn reset_iterator(&mut self, histogram: SimpleHdrHistogram<T>) {
        //TODO
    }
}

#[derive(Debug)]
pub struct RecordedValuesIterator {
    //TODO
    visitedIndex: u32,
}

impl Iterator for RecordedValuesIterator {
    type Item = HistogramIterationValue;
    fn next(&mut self) -> Option<Self::Item> {
        //TODO
        None
    }
}