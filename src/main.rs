mod hdr_histogram;

fn main() {

    let simple_histo = hdr_histogram::simple_hdr_histogram::SimpleHdrHistogram {
        ..Default::default()
    };

    println!("histogram thing: {0}", simple_histo.sub_bucket_mask);

}