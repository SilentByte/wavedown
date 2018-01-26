///
/// Wavedown Utility
/// ****************
/// Copyright (c) 2018 SilentByte
/// <https://github.com/SilentByte/wavedown>
///

#[macro_use]
extern crate clap;

use clap::App;
use clap::Arg;

fn main() {
    let matches = App::new("wavedown")
        .version("1.0")
        .about("Transforms a stream of raw PCM 16bit LE data into \
                a fixed-width waveform representation.")
        .author("Rico A. Beti <rico.beti@silentbyte.com>")
        .arg(Arg::with_name("INPUT")
            .index(1)
            .help("Sets the input file")
            .long("input")
            .short("i")
            .takes_value(true)
            .default_value("-"))
        .arg(Arg::with_name("SAMPLES")
            .help("Sets the number of samples to output")
            .long("samples")
            .short("s")
            .required(true)
            .takes_value(true))
        .get_matches();
}
