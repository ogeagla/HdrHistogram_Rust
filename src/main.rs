mod hdr_histogram;

fn main() {

    let simple_histo = hdr_histogram::simple_hdr_histogram::SimpleHdrHistogram { something: "stuff" };

    println!("histogram thing: {0}", simple_histo.something);

}