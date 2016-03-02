/*

  This struct essentially encapsulates the "instance variables"

*/
pub struct SimpleHdrHistogram {
    pub leading_zeros_count_base: usize,
    pub sub_bucket_mask: u64,
    pub unit_magnitude: u32,
    pub sub_bucket_count: usize,
    pub sub_bucket_half_count: usize,
    pub sub_bucket_half_count_magnitude: u32,
    pub counts: Vec<u64>,
    pub counts_array_length: usize,
    pub normalizing_index_offset: usize,
}


/*

  Implementing this trait (Default) for our struct gives us a nice way to
  initialize an instance using default args instead of having to provide all of them

 */
impl Default for SimpleHdrHistogram {
    fn default () -> SimpleHdrHistogram {
        SimpleHdrHistogram {
            leading_zeros_count_base: 0,
            sub_bucket_mask: 0,
            unit_magnitude: 0,
            sub_bucket_count: 0,
            sub_bucket_half_count: 0,
            sub_bucket_half_count_magnitude: 0,
            counts: Vec::new(),
            counts_array_length: 0,
            normalizing_index_offset: 0,
        }
    }
}

pub trait HistogramBase {

    //FIXME this stuff could be mostly unsigned


    //TODO should be default impl of this trait
    fn record_single_value(&mut self, value: u64) -> Result<(), String>;

    //TODO should be default impl of this trait
    fn counts_array_index(&self, value: u64) -> Result<usize, String>;

    //TODO should be default impl of this trait
    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize;

    //TODO should be default impl of this trait
    fn get_bucket_index(&self, value: u64) -> usize;

    //TODO should be default impl of this trait
    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize;

    fn increment_count_at_index(&mut self, index: usize) -> Result<(), String>;
    fn normalize_index(&self, index: usize, normalizing_index_offset: usize, array_length: usize) ->
        Result<usize, String>;

}

impl HistogramBase for SimpleHdrHistogram {

    fn normalize_index(&self, index: usize, normalizing_index_offset: usize, array_length: usize) ->
Result<usize, String> {

        match normalizing_index_offset {
            0 => Ok(index),
            _ =>
                if ((index > array_length) || (index < 0)) {
                    Err(String::from("index out of covered range"))
                } else {
                    let mut normalized_index: usize = index - normalizing_index_offset;
                    if (normalized_index < 0) {
                        normalized_index += array_length;
                    } else if (normalized_index >= array_length) {
                        normalized_index -=array_length;
                    }
                    Ok(normalized_index)
                }
        }

    }

    fn increment_count_at_index(&mut self, index: usize) -> Result<(), String> {
        let normalized_index =
            self.normalize_index(index, self.normalizing_index_offset, self.counts_array_length);
        match normalized_index {
            Ok(the_index) => {
                self.counts[the_index] += 1;
                Ok(())
            }
            Err(err) =>
                Err(err)
        }
    }


    fn get_sub_bucket_index(&self, value: u64, bucket_index: usize) -> usize {
        let sum = bucket_index + self.unit_magnitude as usize;
        value.rotate_right(sum as u32) as usize
    }

    fn get_bucket_index(&self, value: u64) -> usize {
        let valueOrred = value | self.sub_bucket_mask;
        self.leading_zeros_count_base - (valueOrred.leading_zeros() as usize)
    }

    fn record_single_value(&mut self, value: u64) -> Result<(), String> {

        match self.counts_array_index(value) {
            Ok(counts_index) =>
                match self.increment_count_at_index(counts_index) {
                    Ok(_) =>
                        if true {
                            Ok(())
                        } else {
                            Err(String::from("Could not record single value"))
                        },
                    Err(err) => Err(String::from("Could not increment stuff"))
                },
            Err(err) => Err(String::from("Could not get index"))
        }

    }

    fn counts_array_index(&self, value: u64) -> Result<usize, String> {
        if value < 0 {
            Err(String::from("Histogram recorded values cannot be negative."))
        } else {
            let bucket_index = self.get_bucket_index(value);
            let sub_bucket_index = self.get_sub_bucket_index(value, bucket_index);
            let result = self.counts_array_index_sub(bucket_index, sub_bucket_index);
            Ok(result)
        }
    }

    fn counts_array_index_sub(&self, bucket_index: usize, sub_bucket_index: usize) -> usize {
        assert!(sub_bucket_index < self.sub_bucket_count);
        assert!(bucket_index == 0 || (sub_bucket_index >= self.sub_bucket_half_count));

        let bucket_base_index = (bucket_index + 1) << self.sub_bucket_half_count_magnitude;
        let offset_in_bucket = sub_bucket_index - self.sub_bucket_half_count;

        bucket_base_index + offset_in_bucket
    }
}

#[test]
fn can_record_single_value() {
    let the_hist = SimpleHdrHistogram { ..Default::default() };
    let result = the_hist.record_single_value(99);

    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }
}

#[test]
fn can_compute_counts_array_index() {
    let the_hist = SimpleHdrHistogram { ..Default::default() };
    let result = the_hist.counts_array_index(99);

    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not compute counts array index because error: {}", err))
    }
}

#[test]
fn can_get_bucket_index() {
    let the_hist = init_histo();
    let result = the_hist.get_bucket_index(99);
    assert_eq!(result, 0)
}

#[test]
fn can_get_sub_bucket_index() {
    let the_hist = SimpleHdrHistogram { ..Default::default() };
    let result = the_hist.get_sub_bucket_index(99, 1);
    assert_eq!(result, 0)
}