use hdr_histogram::simple_hdr_histogram::*;

#[derive(Debug)]
#[derive(Clone)]
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
    fn set(&mut self,
            value_iterated_to: u64,
            value_iterated_from: u64,
            count_at_value_iterated_to: u64,
            count_added_in_this_iteration_step: u64,
            total_count_to_this_value: u64,
            total_value_to_this_value: u64,
            percentile: f64,
            percentile_level_iterated_to: f64,
            integer_to_double_value_conversion_ratio: f64) {
        self.value_iterated_to = value_iterated_to;
        self.value_iterated_from = value_iterated_from;
        self.count_at_value_iterated_to = count_at_value_iterated_to;
        self.count_added_in_this_iteration_step = count_added_in_this_iteration_step;
        self.total_count_to_this_value = total_count_to_this_value;
        self.total_value_to_this_value = total_value_to_this_value;
        self.percentile = percentile;
        self.percentile_level_iterated_to = percentile_level_iterated_to;
        self.integer_to_double_value_conversion_ratio = integer_to_double_value_conversion_ratio;
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
    count_at_this_value: T,
    fresh_sub_bucket: bool,
    current_iteration_value: HistogramIterationValue,
    integer_to_double_value_conversion_ratio: f64,
    visited_index: i32,
}

impl<T: HistogramCount> BaseHistogramIterator<T> {
    fn reset_iterator(mut self, histogram: SimpleHdrHistogram<T>) {
        self.histogram = histogram;
        self.saved_histogram_total_raw_count = self.histogram.get_count();
        self.array_total_count = self.histogram.get_count();
        self.integer_to_double_value_conversion_ratio = 0.0; //TODO should be self.histogram.integer_to_double_value_conversion_ratio
        self.current_index = 0;
        self.current_value_at_index = 0;
        self.next_value_at_index = 1 << self.histogram.get_unit_magnitude();
        self.prev_value_iterated_to = 0;
        self.total_count_to_prev_index = 0;
        self.total_count_to_current_index = 0;
        self.total_value_to_current_index = 0;
        self.count_at_this_value = T::zero();
        self.fresh_sub_bucket = true;
        self.visited_index = -1;
        self.current_iteration_value.reset();
    }
    fn exhausted_sub_buckets(&self) -> bool {
        self.current_index >= 1000 //TODO should be self.histogram.get_counts_array_length()
    }

    pub fn reached_iter_level(&self) -> bool {
        let current_count = match self.histogram.get_count_at_index(self.current_index) {
            Err(err) => 0,
            Ok(the_count) => match the_count.to_u64() {
                None => 0,
                Some(the_val) => the_val
            }
        };
        (current_count != 0) && (self.visited_index != self.current_index as i32)
    }

    pub fn get_value_iterated_to(&self) -> u64 {
        self.histogram.highest_equivalent_value(self.current_value_at_index)
    }

    pub fn increment_iteration_level(&mut self) {
        self.visited_index = self.current_index as i32;
    }

    pub fn increment_sub_bucket(&mut self) {
        self.fresh_sub_bucket = true;
        self.current_index += 1;
        self.current_value_at_index = self.histogram.value_from_index(self.current_index);
        self.next_value_at_index = self.histogram.value_from_index(self.current_index + 1);
    }
}

impl <T: HistogramCount> Iterator for BaseHistogramIterator<T> {

    type Item = HistogramIterationValue;
    fn next(&mut self) -> Option<Self::Item> {
        //combine Java's hasNext() and next()
        //first check if has next
        if self.histogram.get_count() != self.saved_histogram_total_raw_count {
            //in Java, this threw a ConcurrentModificationException
            return None
        }
        if self.total_count_to_current_index >= self.array_total_count {
            //this means hasNext() returns false
            return None
        }

        fn get_percentile_iterated_to(total_count_to_current_index: f64, array_total_count: f64) -> f64 {
            (100.0 * total_count_to_current_index) / array_total_count
        }

        while ! self.exhausted_sub_buckets() {
            self.count_at_this_value = match self.histogram.get_count_at_index(self.current_index) {
                Ok(val) => val,
                Err(err) => T::zero()
            };
            if self.fresh_sub_bucket {
                let count_u64 = match self.count_at_this_value.to_u64() {
                    None => 0,
                    Some(the_long) => the_long
                };
                self.total_count_to_current_index += count_u64;
                let highest_eq_val = self.histogram.highest_equivalent_value(self.current_value_at_index);
                self.total_value_to_current_index += count_u64 * highest_eq_val;
                self.fresh_sub_bucket = false;
                if self.reached_iter_level() {
                    let value_iterated_to = self.get_value_iterated_to();
                    self.current_iteration_value.set(
                        value_iterated_to,
                        self.prev_value_iterated_to,
                        match self.count_at_this_value.to_u64() {
                            None => 0,
                            Some(the_count) => the_count
                        },
                        (self.total_count_to_current_index - self.total_count_to_prev_index),
                        self.total_count_to_current_index,
                        self.total_value_to_current_index,
                        ((100.0 * self.total_count_to_current_index as f64) / self.array_total_count as f64),
                        get_percentile_iterated_to(self.total_count_to_current_index as f64, self.array_total_count as f64),
                        self.integer_to_double_value_conversion_ratio);
                    self.prev_value_iterated_to = value_iterated_to;
                    self.total_count_to_prev_index = self.total_count_to_current_index;
                    self.increment_iteration_level();

                    if self.histogram.get_count() != self.saved_histogram_total_raw_count {
                        //TODO this is bad
                    }

                    return Some(self.current_iteration_value.clone())
                }
                self.increment_sub_bucket();
            }
        }
        None
    }
}


#[derive(Debug)]
pub struct RecordedValuesIterator {
    //TODO
    visited_index: u32,
}

impl Iterator for RecordedValuesIterator {
    type Item = HistogramIterationValue;
    fn next(&mut self) -> Option<Self::Item> {
        //TODO
        None
    }
}