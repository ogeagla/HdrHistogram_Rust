pub trait Histogram {
    //instance method
    fn inst_func(&self) -> &'static str;

    //static method
    fn static_fun(thing: &'static str) -> Self;
}

pub struct SimpleHdrHistogram { pub something: &'static str }

impl Histogram for SimpleHdrHistogram {

    fn inst_func(&self) -> &'static str {
        "hello from instance"
    }

    fn static_fun(thing: &'static str) -> SimpleHdrHistogram {
        SimpleHdrHistogram { something: "stuff" }
    }
}