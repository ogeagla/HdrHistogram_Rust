use hdr_histogram::simple_hdr_histogram::*;

#[derive(Debug)]
#[derive(Clone)]
pub struct HistogramIterationValue<T: HistogramCount> {
    value_iterated_to: u64,
    value_iterated_from: u64,
    count_at_value_iterated_to: T,
    count_added_in_this_iteration_step: u64, // TODO can this be T?
    total_count_to_this_value: u64, // TODO Generify to allow for bigint?
    total_value_to_this_value: u64, // TODO Generify to allow for bigint?
    percentile: f64,
    percentile_level_iterated_to: f64,
    integer_to_double_value_conversion_ratio: f64,
}

impl<T: HistogramCount> Default for HistogramIterationValue<T> {
    fn default() -> HistogramIterationValue<T> {
        HistogramIterationValue {
            value_iterated_to: 0,
            value_iterated_from: 0,
            count_at_value_iterated_to: T::zero(),
            count_added_in_this_iteration_step: 0,
            total_count_to_this_value: 0,
            total_value_to_this_value: 0,
            percentile: 0.0,
            percentile_level_iterated_to: 0.0,
            integer_to_double_value_conversion_ratio: 0.0,
        }
    }
}

impl<T: HistogramCount> HistogramIterationValue<T> {
    fn reset(&mut self) {
        *self = HistogramIterationValue { ..HistogramIterationValue::default() };
    }
    fn set(&mut self,
    value_iterated_to: u64,
    value_iterated_from: u64,
    count_at_value_iterated_to: T,
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

    pub fn get_value_iterated_to(&self) -> u64 {
        self.value_iterated_to
    }

    pub fn get_count_at_value_iterated_to(&self) -> T {
        self.count_at_value_iterated_to
    }
}

impl<'a, T: HistogramCount> IntoIterator for RecordedValues<'a, T> {
    type Item = HistogramIterationValue<T>;
    type IntoIter = BaseHistogramIterator<'a, T, RecordedValuesStrategy>;

    fn into_iter(self) -> Self::IntoIter {
        BaseHistogramIterator {
            histogram: self.histo,
            strategy: RecordedValuesStrategy { visited_index: -1 },
            saved_histogram_total_raw_count: self.histo.get_count(),
            array_total_count: self.histo.get_count(),
            integer_to_double_value_conversion_ratio: 0.0, //TODO should be histogram.integer_to_double_value_conversion_ratio
            current_index: 0,
            current_value_at_index: 0,
            next_value_at_index: 1 << self.histo.get_unit_magnitude(),
            prev_value_iterated_to: 0,
            total_count_to_prev_index: 0,
            total_count_to_current_index: 0,
            total_value_to_current_index: 0,
            count_at_this_value: T::zero(),
            fresh_sub_bucket: true,
            visited_index: -1,
            current_iteration_value: HistogramIterationValue::default()
        }
    }
}

pub struct RecordedValuesStrategy {
    visited_index: i32
}

impl<'a, T: HistogramCount + 'a> IterationStrategy<'a, T> for RecordedValuesStrategy {

    fn increment_iteration_level(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) {
        self.visited_index = iter.current_index as i32;
    }

    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        let current_count: T = match iter.histogram.get_count_at_index(iter.current_index) {
            Err(err) => {
                println!("error: {}", err);
                T::zero()}, // TODO
            Ok(the_count) => the_count
        };
        // cast is safe; count indexes << 2^32
        (current_count != T::zero()) && (self.visited_index != iter.current_index as i32)
    }
}

// this is really a recorded value iterator mashed together with its base class
impl<'a, T: HistogramCount + 'a, S: IterationStrategy<'a, T>> BaseHistogramIterator<'a, T, S> {

    fn reset_iterator(&mut self, histogram: &'a SimpleHdrHistogram<T>) {
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
        self.current_iteration_value.reset();

        self.visited_index = -1;
    }

    fn has_next(&self) -> bool {
        // TODO is this even possible with the borrow checker?
        if self.histogram.get_count() != self.saved_histogram_total_raw_count {
            //in Java, this threw a ConcurrentModificationException
            return false
        }
        if self.total_count_to_current_index >= self.array_total_count {
            //this means hasNext() returns false
            return false
        }

        true
    }


    fn get_percentile_iterated_to(&self) -> f64 {
        (100.0 * self.total_count_to_current_index as f64) / self.array_total_count as f64
    }

    fn get_value_iterated_to(&self) -> u64 {
        self.histogram.highest_equivalent_value(self.current_value_at_index)
    }

    fn exhausted_sub_buckets(&self) -> bool {
        self.current_index >= self.histogram.counts.len()
    }

    fn increment_sub_bucket(&mut self) {
        self.fresh_sub_bucket = true;
        self.current_index += 1;
        self.current_value_at_index = self.histogram.value_from_index(self.current_index);
        // TODO what if this overflows the index?
        self.next_value_at_index = self.histogram.value_from_index(self.current_index + 1);
    }

}

impl<'a, T: HistogramCount + 'a, S: IterationStrategy<'a, T>> Iterator for BaseHistogramIterator<'a, T, S> {
    type Item = HistogramIterationValue<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if ! self.has_next() {
            return None
        }

        while ! self.exhausted_sub_buckets() {
            self.count_at_this_value = match self.histogram.get_count_at_index(self.current_index) {
                Ok(val) => val,
                Err(err) => T::zero() // TODO handle error
            };
            if self.fresh_sub_bucket {
                // all count types can become u64
                let count_u64 = self.count_at_this_value.to_u64().unwrap();
                self.total_count_to_current_index += count_u64;
                let highest_eq_val = self.histogram.highest_equivalent_value(self.current_value_at_index);
                self.total_value_to_current_index += count_u64 * highest_eq_val;
                self.fresh_sub_bucket = false;
            }

            if self.strategy.reached_iteration_level(self) {
                let value_iterated_to = self.get_value_iterated_to();
                let pctile_iterated_to = self.get_percentile_iterated_to();

                self.current_iteration_value.set(
                    value_iterated_to,
                    self.prev_value_iterated_to,
                    self.count_at_this_value,
                    (self.total_count_to_current_index - self.total_count_to_prev_index),
                    self.total_count_to_current_index,
                    self.total_value_to_current_index,
                    ((100.0 * self.total_count_to_current_index as f64) / self.array_total_count as f64),
                    pctile_iterated_to,
                    self.integer_to_double_value_conversion_ratio);
                self.prev_value_iterated_to = value_iterated_to;
                self.total_count_to_prev_index = self.total_count_to_current_index;
                
                self.strategy.increment_iteration_level(self);

                if self.histogram.get_count() != self.saved_histogram_total_raw_count {
                    //TODO this is bad. Is this possible w/ borrow checker?
                }

                return Some(self.current_iteration_value.clone())
            }
            self.increment_sub_bucket();
        }
        None // TODO what hits here?
    }

}
