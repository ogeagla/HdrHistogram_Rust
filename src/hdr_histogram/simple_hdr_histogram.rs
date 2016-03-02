pub trait Histogram {
    //instance method example
    fn inst_func(&self) -> &'static str;

    //static method example
    fn static_fun(thing: &'static str) -> Self;

    fn record_single_value(&self, value: i64) -> Result<(), String>;


}

pub struct SimpleHdrHistogram { pub something: &'static str }

impl Histogram for SimpleHdrHistogram {

    fn inst_func(&self) -> &'static str {
        "hello from instance"
    }

    fn static_fun(thing: &'static str) -> SimpleHdrHistogram {
        SimpleHdrHistogram { something: "stuff" }
    }

    fn record_single_value(&self, value: i64) -> Result<(), String> {
        if true {
            Ok(())
        } else {
            Err(String::from("Could not record single value"))
        }
    }
}

#[test]
fn it_works() {
    let the_hist = SimpleHdrHistogram { something: "nothing" };
    let result = the_hist.record_single_value(99);

    match result {
        Ok(_) => (),
        Err(err) => panic!(format!("could not add single record to histogram because error: {}", err))
    }
}