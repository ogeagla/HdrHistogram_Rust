use std::mem;

use hdr_histogram::simple_hdr_histogram::*;

#[derive(Debug)]
#[derive(Clone)]
pub struct HistogramIterationValue<T: HistogramCount> {
    // TODO add tests for other fields we want to expose and mark public
    pub value_iterated_to: u64,
    value_iterated_from: u64,
    pub count_at_value_iterated_to: T,
    // many counts may be covered in one step, so use largest type
    pub count_added_in_this_iteration_step: u64,
    total_count_to_this_value: u64, // TODO Generify to allow for bigint?
    total_value_to_this_value: u64, // TODO Generify to allow for bigint?
    /// The percentile at the current position
    pub percentile: f64,
    /// The percentile intended to iterate to. This can be different from percentile if, for
    /// instance, we intend to iterate to the 90th %ile but value distribution does not allow us
    /// to exactly hit 90%.
    pub percentile_level_iterated_to: f64,
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
}

impl<'a, T: HistogramCount> IntoIterator for RecordedValues<'a, T> {
    type Item = HistogramIterationValue<T>;
    type IntoIter = BaseHistogramIterator<'a, T, RecordedValuesStrategy>;

    fn into_iter(self) -> Self::IntoIter {
        BaseHistogramIterator::new(self.histo, RecordedValuesStrategy { visited_index: -1 })
    }
}

impl<'a, T: HistogramCount> IntoIterator for AllValues<'a, T> {
    type Item = HistogramIterationValue<T>;
    type IntoIter = BaseHistogramIterator<'a, T, AllValuesStrategy>;

    fn into_iter(self) -> Self::IntoIter {
        BaseHistogramIterator::new(self.histo, AllValuesStrategy { visited_index: -1 })
    }
}

impl<'a, T: HistogramCount> IntoIterator for LogarithmicValues<'a, T> {
    type Item = HistogramIterationValue<T>;
    type IntoIter = BaseHistogramIterator<'a, T, LogarithmicValuesStrategy>;

    fn into_iter(self) -> Self::IntoIter {
        let first_step_highest_value = self.value_units_in_first_bucket - 1;
        BaseHistogramIterator::new(self.histo, LogarithmicValuesStrategy {
            log_base: self.log_base,
            next_value_reporting_level: self.value_units_in_first_bucket,
            current_step_highest_value_reporting_level: first_step_highest_value,
            current_step_lowest_value_reporting_level:
                self.histo.lowest_equivalent_value(first_step_highest_value)
        })
    }
}

impl<'a, T: HistogramCount> IntoIterator for LinearValues<'a, T> {
    type Item = HistogramIterationValue<T>;
    type IntoIter = BaseHistogramIterator<'a, T, LinearValuesStrategy>;

    fn into_iter(self) -> Self::IntoIter {
        let first_step_highest_value = self.value_units_per_bucket - 1;
        BaseHistogramIterator::new(self.histo, LinearValuesStrategy {
            value_units_per_bucket: self.value_units_per_bucket,
            current_step_highest_value_reporting_level: first_step_highest_value,
            current_step_lowest_value_reporting_level:
            self.histo.lowest_equivalent_value(first_step_highest_value)
        })
    }
}

impl<'a, T: HistogramCount> IntoIterator for Percentiles<'a, T> {
    type Item = HistogramIterationValue<T>;
    type IntoIter = BaseHistogramIterator<'a, T, PercentilesStrategy>;

    fn into_iter(self) -> Self::IntoIter {
        BaseHistogramIterator::new(self.histo, PercentilesStrategy {
            percentile_ticks_per_half_distance: self.percentile_ticks_per_half_distance,
            percentile_level_to_iterate_to: 0.0,
            percentile_level_to_iterate_from: 0.0,
            reached_last_recorded_value: false
        })
    }
}

/// Iterates through the values with non-zero counts. When the equivalent value range > 1, the
/// highest equivalent value is used.
pub struct RecordedValuesStrategy {
    visited_index: i32
}

impl<'a, T: HistogramCount + 'a> IterationStrategy<'a, T> for RecordedValuesStrategy {

    fn increment_iteration_level(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) {
        // cast is safe; count indexes << 2^32
        self.visited_index = iter.current_index as i32;
    }

    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        let current_count: T = match iter.histogram.get_count_at_index(iter.current_index) {
            Err(err) => {
                println!("error: {}", err);
                T::zero()}, // TODO
            Ok(the_count) => the_count
        };
        // detects when we enter the main iteration loop for the first time after having previously
        // returned out of the loop
        (current_count != T::zero()) && (self.visited_index != iter.current_index as i32)
    }

    fn dummy() -> Self {
        RecordedValuesStrategy {
            visited_index: i32::min_value()
        }
    }
}

pub struct AllValuesStrategy {
    visited_index: i32
}

impl<'a, T: HistogramCount + 'a> IterationStrategy<'a, T> for AllValuesStrategy {

    fn increment_iteration_level(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) {
        // cast is safe; count indexes << 2^32
        self.visited_index = iter.current_index as i32;
    }

    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        self.visited_index != iter.current_index as i32
    }

    fn allow_further_iteration(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        // Unlike other iterators AllValues is only done when we've exhausted the indices:
        iter.current_index < (iter.histogram.counts.len() - 1)
    }

    fn dummy() -> Self {
        AllValuesStrategy {
            visited_index: i32::min_value()
        }
    }
}

pub struct LogarithmicValuesStrategy {
    // Java impl used double for this; simpler to stick with integer until we know we need double
    log_base: u64,
    next_value_reporting_level: u64,
    current_step_highest_value_reporting_level: u64,
    current_step_lowest_value_reporting_level: u64
}

impl<'a, T: HistogramCount + 'a> IterationStrategy<'a, T> for LogarithmicValuesStrategy {

    fn increment_iteration_level(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) {
        self.next_value_reporting_level *= self.log_base;
        self.current_step_highest_value_reporting_level = self.next_value_reporting_level - 1;
        self.current_step_lowest_value_reporting_level =
            iter.histogram.lowest_equivalent_value(self.current_step_highest_value_reporting_level);
    }

    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        // emit current position if the value fits within the current reporting range or we've
        // reached the last element while looking for a big enough value. This last would happen
        // if no values are big enough to reach the next log bucket.
        (iter.current_value_at_index >= self.current_step_lowest_value_reporting_level)
            || (iter.current_index >= iter.histogram.counts.len() - 1)
    }

    fn allow_further_iteration(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        if self._default_allow_further_iteration(iter) {
            return true;
        }

        // failed default check, so count to this point would be equal to total count. Thus,
        // next sub bucket would be empty, but we're not done iterating if the next reporting level
        // would fall below the value contained in the next (nonexistent) sub bucket, so we need
        // to double one more time
        return iter.histogram.lowest_equivalent_value(self.next_value_reporting_level)
            < iter.next_value_at_index
    }

    fn value_iterated_to(&self, _: &BaseHistogramIterator<'a, T, Self>) -> u64 {
        self.current_step_highest_value_reporting_level
    }

    fn dummy() -> Self {
        LogarithmicValuesStrategy {
            log_base: 0,
            next_value_reporting_level: 0,
            current_step_highest_value_reporting_level: 0,
            current_step_lowest_value_reporting_level: 0
        }
    }
}

pub struct LinearValuesStrategy {
    value_units_per_bucket: u64,
    current_step_highest_value_reporting_level: u64,
    current_step_lowest_value_reporting_level: u64
}

impl<'a, T: HistogramCount + 'a> IterationStrategy<'a, T> for LinearValuesStrategy {

    fn increment_iteration_level(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) {
        self.current_step_highest_value_reporting_level += self.value_units_per_bucket;
        self.current_step_lowest_value_reporting_level =
            iter.histogram.lowest_equivalent_value(self.current_step_highest_value_reporting_level);
    }

    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        // emit current position if the value fits within the current reporting range or we've
        // reached the last element while looking for a big enough value. This last would happen
        // if no values are big enough to reach the next log bucket.
        (iter.current_value_at_index >= self.current_step_lowest_value_reporting_level)
            || (iter.current_index >= iter.histogram.counts.len() - 1)
    }

    fn allow_further_iteration(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        if self._default_allow_further_iteration(iter) {
            return true;
        }

        // failed default check, so count to this point would be equal to total count. Thus,
        // next sub bucket would be empty, but we're not done iterating if the next reporting level
        // would fall below the value contained in the next (nonexistent) sub bucket, so we need
        // to expose one more linear bucket.
        self.current_step_highest_value_reporting_level + 1 < iter.next_value_at_index
    }

    fn value_iterated_to(&self, _: &BaseHistogramIterator<'a, T, Self>) -> u64 {
        self.current_step_highest_value_reporting_level
    }

    fn dummy() -> Self {
        LinearValuesStrategy {
            value_units_per_bucket: 0,
            current_step_highest_value_reporting_level: 0,
            current_step_lowest_value_reporting_level: 0
        }
    }
}

pub struct PercentilesStrategy {
    percentile_ticks_per_half_distance: u32,
    percentile_level_to_iterate_to: f64,
    percentile_level_to_iterate_from: f64,
    reached_last_recorded_value: bool
}

impl<'a, T: HistogramCount + 'a> IterationStrategy<'a, T> for PercentilesStrategy {

    fn increment_iteration_level(&mut self, _: &BaseHistogramIterator<'a, T, Self>) {
        self.percentile_level_to_iterate_from = self.percentile_level_to_iterate_to;

        // To calculate the delta to add on at the current iteration, we want to know how many
        // iterations at the current scale it would take to go from 0 to 100.
        // By definition, it should take percentile_ticks_per_half_distance to go half the
        // remaining distance, so the ticks to go 0-100 is
        // 2 * pctile_ticks_per_half * (multiples of remaining distance that fit in 0-100).
        //
        // However, this will give a "natural" half-distance behavior, where each iteration is
        // slightly smaller than the one preceding it. To make it easier for users to reason about,
        // it would be nice if the iteration size would stay the same throughout the entire first
        // half, then divide in half and stay the same for the next quarter, etc. To do this,
        // instead of using simply the number of times that the remaining distance will fit, we use
        // the greatest power of 2 smaller than that. This will exhibit the desired behavior of
        // "the same every time until it can double".

        // number of times the remaining percentile distance would fit into 100.0
        let multiples_of_remaining_distance: f64 = 100.0/(100.0 - self.percentile_level_to_iterate_to);
        // 2x the largest power of 2 that's smaller than the number above (the + 1 power of 2
        // handles the doubling needed because we have ticks per *half*)
        let multiples_pwr2: u32 = 2_u32.pow((multiples_of_remaining_distance.ln() / 2_f64.ln()) as u32 + 1);
        // total number of ticks
        let pctile_ticks: u32 = self.percentile_ticks_per_half_distance * multiples_pwr2;
        // add on the per-tick delta
        self.percentile_level_to_iterate_to += 100.0 / pctile_ticks as f64;
    }

    fn reached_iteration_level(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        if iter.count_at_this_value == T::zero() {
            return false;
        }

        let current_percentile: f64 = (100.0 * iter.total_count_to_current_index as f64) / iter.array_total_count as f64;
        current_percentile >= self.percentile_level_to_iterate_to
    }

    fn allow_further_iteration(&mut self, iter: &BaseHistogramIterator<'a, T, Self>) -> bool {
        if self._default_allow_further_iteration(iter) {
            return true;
        }

        // want to have one last step to 100%
        if !self.reached_last_recorded_value && (iter.array_total_count > 0) {
            self.percentile_level_to_iterate_to = 100.0;
            self.reached_last_recorded_value = true;
            return true;
        }

        return false;
    }

    fn percentile_iterated_to(&self, iter: &BaseHistogramIterator<'a, T, Self>) -> f64 {
        self.percentile_level_to_iterate_to
    }

    fn dummy() -> Self {
        PercentilesStrategy {
            percentile_ticks_per_half_distance: 0,
            percentile_level_to_iterate_to: 0.0,
            percentile_level_to_iterate_from: 0.0,
            reached_last_recorded_value: false
        }
    }
}

// this is really a recorded value iterator mashed together with its base class
impl<'a, T: HistogramCount + 'a, S: IterationStrategy<'a, T>> BaseHistogramIterator<'a, T, S> {

    fn new(histo: &'a SimpleHdrHistogram<T>, strategy: S) -> BaseHistogramIterator<'a, T, S> {
        BaseHistogramIterator {
            histogram: histo,
            strategy: strategy,
            saved_histogram_total_raw_count: histo.get_count(),
            array_total_count: histo.get_count(),
            integer_to_double_value_conversion_ratio: 0.0, //TODO should be histogram.integer_to_double_value_conversion_ratio
            current_index: 0,
            current_value_at_index: 0,
            next_value_at_index: 1 << histo.get_unit_magnitude(),
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

    fn exhausted_sub_buckets(&self) -> bool {
        self.current_index >= self.histogram.counts.len()
    }

    fn increment_sub_bucket(&mut self) {
        self.fresh_sub_bucket = true;
        self.current_index += 1;
        self.current_value_at_index = self.histogram.value_from_index(self.current_index);
        // This only calculates what the value would be, so it doesn't hit the counts array, and
        // therefore won't overflow when we try the last index
        self.next_value_at_index = self.histogram.value_from_index(self.current_index + 1);
    }

}

impl<'a, T: HistogramCount + 'a, S: IterationStrategy<'a, T>> Iterator for BaseHistogramIterator<'a, T, S> {
    // TODO should this be a ref?
    type Item = HistogramIterationValue<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // borrow checker won't let us pass an immutable borrow of self to i_i_s when
        // self.strategy is borrowed as mutable, and we can't pass a mutable borrow of self
        // because that would be two mutable borrows. Thus, we temporarily drop a dummy
        // value in so that the strategy isn't owned by self.
        let mut s = mem::replace(&mut self.strategy, S::dummy());
        let proceed = s.allow_further_iteration(self);
        self.strategy = s;

        if ! proceed {
            return None
        }

        while ! self.exhausted_sub_buckets() {
            self.count_at_this_value = match self.histogram.get_count_at_index(self.current_index) {
                Ok(val) => val,
                Err(err) => T::one() + T::one() + T::one() // TODO handle error
            };
            if self.fresh_sub_bucket {
                // all count types can become u64
                let count_u64 = self.count_at_this_value.as_u64();
                self.total_count_to_current_index += count_u64;
                let highest_eq_val = self.histogram.highest_equivalent_value(self.current_value_at_index);
                self.total_value_to_current_index += count_u64 * highest_eq_val;
                self.fresh_sub_bucket = false;
            }

            if self.strategy.reached_iteration_level(self) {
                let value_iterated_to = self.strategy.value_iterated_to(self);
                let pctile_iterated_to = self.strategy.percentile_iterated_to(self);

                self.current_iteration_value.set(
                    value_iterated_to,
                    self.prev_value_iterated_to,
                    self.count_at_this_value,
                    (self.total_count_to_current_index - self.total_count_to_prev_index),
                    self.total_count_to_current_index,
                    self.total_value_to_current_index,
                    (100.0 * self.total_count_to_current_index as f64) / self.array_total_count as f64,
                    pctile_iterated_to,
                    self.integer_to_double_value_conversion_ratio);
                self.prev_value_iterated_to = value_iterated_to;
                self.total_count_to_prev_index = self.total_count_to_current_index;

                // more borrow checker shenanigans
                let mut s = mem::replace(&mut self.strategy, S::dummy());
                s.increment_iteration_level(self);
                self.strategy = s;

                // java impl checked for count change here, but borrow checker protects us

                // TODO could we just expose a ref to the current value here?
                return Some(self.current_iteration_value.clone())
            }
            self.increment_sub_bucket();
        }
        None // TODO what hits here?
    }

}
