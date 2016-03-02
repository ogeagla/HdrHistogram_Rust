mod hdr_histogram;

fn main() {

    let simple_histo = hdr_histogram::simple_hdr_histogram::SimpleHdrHistogram {
        leading_zeros_count_base: 0,
        sub_bucket_mask: 0
    };

    println!("histogram thing: {0}", simple_histo.sub_bucket_mask);

}