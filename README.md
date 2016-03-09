# HdrHistogram_Rust
HDR Histogram Rust Port

## WTF is this
 - [HDR Histogram](http://hdrhistogram.org/)
 - [Java reference implementation on which this port is based](https://github.com/HdrHistogram/HdrHistogram)

 See the Java implementation (especially `AbstractHistogram` javadocs) for info on how the data structure works.

## How to build
```
cargo build
```

## How to run tests
```
cargo test
```

## How to build documentation
```
cargo doc
```

## Modules
 - `simple_hdr_histogram` - Base HDR Histogram implementation
