pub trait Histogram {

    //TODO should be default impl of this trait
    fn record_single_value(&self, value: i64) -> Result<(), String>;

    //TODO should be default impl of this trait
    fn counts_array_index(&self, value: i64) -> Result<i32, String>;

    //TODO should be default impl of this trait
    fn get_bucket_index(&self, value: i64) -> i32;

    //TODO should be default impl of this trait
    fn get_sub_bucket_index(&self, value: i64, bucket_index: i32) -> i32;

}

pub struct SimpleHdrHistogram {
    pub leading_zeros_count_base: i32,
    pub sub_bucket_mask: i64,
    pub unit_magnitude: i32,
}

impl Default for SimpleHdrHistogram {
    fn default () -> SimpleHdrHistogram {
        SimpleHdrHistogram {
            leading_zeros_count_base: 0,
            sub_bucket_mask: 0,
            unit_magnitude: 0
        }
    }
}

impl Histogram for SimpleHdrHistogram {

    fn get_sub_bucket_index(&self, value: i64, bucket_index: i32) -> i32 {
        let sum = bucket_index + self.unit_magnitude;
        value.rotate_right(sum as u32) as i32
    }

    fn get_bucket_index(&self, value: i64) -> i32 {
        let valueOrred = value | self.sub_bucket_mask;
        self.leading_zeros_count_base - valueOrred.leading_zeros() as i32
    }

    fn record_single_value(&self, value: i64) -> Result<(), String> {

        let counts_index = self.counts_array_index(value);

        if true {
            Ok(())
        } else {
            Err(String::from("Could not record single value"))
        }

    }

    fn counts_array_index(&self, value: i64) -> Result<i32, String> {

        if (value < 0) {
            Err(String::from("Histogram recorded values cannot be negative."))
        } else {
            Ok(0)
        }
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
    let the_hist = SimpleHdrHistogram { ..Default::default() };
    let result = the_hist.get_bucket_index(99);
    assert_eq!(result, 0)
}

#[test]
fn can_get_sub_bucket_index() {
    let the_hist = SimpleHdrHistogram { ..Default::default() };
    let result = the_hist.get_sub_bucket_index(99, 1);
    assert_eq!(result, 0)
}