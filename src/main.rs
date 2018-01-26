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

use std::mem;
use std::io::{stdin, stdout};
use std::io::{Read, Write, BufWriter};
use std::fs::File;

type AppError = Result<(), String>;

#[derive(Debug)]
enum OutputType {
    Byte,
    Short,
    Float,
}

#[derive(Debug)]
struct AppArgs {
    input: String,
    output: String,
    samples: usize,
    output_type: OutputType,
    precision: usize,
}

#[derive(Debug, Copy, Clone)]
struct MinMax<T> {
    min: T,
    max: T,
}

impl<T> MinMax<T> {
    fn new(min: T, max: T) -> Self {
        MinMax {
            min,
            max,
        }
    }
}

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
        .arg(Arg::with_name("OUTPUT")
            .index(2)
            .help("Sets the output file")
            .long("output")
            .short("o")
            .takes_value(true)
            .default_value("-"))
        .arg(Arg::with_name("SAMPLES")
            .help("Sets the number of samples to output")
            .long("samples")
            .short("s")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("TYPE")
            .help("Sets the type of the output values")
            .long("type")
            .short("t")
            .takes_value(true)
            .possible_values(&["byte", "short", "float"])
            .default_value("short"))
        .arg(Arg::with_name("PRECISION")
            .help("Sets the floating point precision")
            .long("precision")
            .short("p")
            .takes_value(true)
            .default_value("7"))
        .get_matches();

    let result = run(AppArgs {
        input: matches.value_of("INPUT").unwrap().into(),
        output: matches.value_of("OUTPUT").unwrap().into(),
        samples: value_t_or_exit!(matches.value_of("SAMPLES"), usize),
        output_type: match matches.value_of("TYPE").unwrap().to_lowercase().as_ref() {
            "byte" => OutputType::Byte,
            "short" => OutputType::Short,
            "float" => OutputType::Float,
            _ => panic!("Invalid output type.")
        },
        precision: clamp(0, 7, value_t_or_exit!(matches.value_of("PRECISION"), usize)),
    });

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}

fn run(args: AppArgs) -> AppError {
    if args.samples == 0 {
        return Ok(());
    }

    let mut input_stream: Box<Read> = match args.input.as_ref() {
        "-" => Box::new(stdin()),
        input @ _ => Box::new(File::open(input)
            .map_err(|_| "File input stream could not be read.")?)
    };

    let mut output_stream: Box<Write> = match args.output.as_ref() {
        "-" => Box::new(BufWriter::new(stdout())),
        output @ _ => Box::new(File::create(output)
            .map_err(|_| "File output stream could not be created.")?)
    };

    let pcm_data = read_pcm_from_stream(&mut *input_stream)?;
    let pcm_data_count = pcm_data.len();

    let sample_count = args.samples;
    let subsample_count = pcm_data_count / sample_count;

    if subsample_count == 0 {
        return Err("Not enough PCM data to transform into \
                    the specified number of samples.".into());
    }

    for sample_index in 0..sample_count {
        let mut local_peak = MinMax::<i16>::new(0, 0);
        let mut total_peak = MinMax::<i64>::new(0, 0);

        for subsample_index in 0..subsample_count {
            let sample = pcm_data[sample_index * subsample_count + subsample_index];

            if sample < local_peak.min {
                local_peak.min = sample;
            } else if sample > local_peak.max {
                local_peak.max = sample;
            }

            total_peak.min += local_peak.min as i64;
            total_peak.max += local_peak.max as i64;
        }

        let average_peak = MinMax {
            min: (total_peak.min / subsample_count as i64) as i16,
            max: (total_peak.max / subsample_count as i64) as i16,
        };

        output_stream
            .write(format_peak(&average_peak, &args).as_bytes())
            .map_err(|_| "Could not write to output.".to_owned())?;
    }

    Ok(())
}

fn clamp<T>(lower: T, upper: T, value: T) -> T
    where T: PartialOrd
{
    if value < lower {
        lower
    } else if value > upper {
        upper
    } else {
        value
    }
}

fn read_pcm_from_stream(stream: &mut Read) -> Result<Vec<i16>, String> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();

    let length = buffer.len();
    if length % 2 != 0 {
        return Err("PCM stream length is not divisible by 2.".into());
    }

    let ptr = buffer.as_mut_ptr();
    let capacity = buffer.capacity();

    unsafe {
        mem::forget(buffer);
        Ok(Vec::from_raw_parts(ptr as *mut i16,
                               length / 2,
                               capacity / 2))
    }
}

fn format_peak(peak: &MinMax<i16>, args: &AppArgs) -> String {
    match args.output_type {
        OutputType::Byte => format!("{} {}\n",
                                    peak.min / (0xFFFF / 0xFF) as i16,
                                    peak.max / (0xFFFF / 0xFF) as i16),

        OutputType::Short => format!("{} {}\n",
                                     peak.min,
                                     peak.max),

        OutputType::Float => format!("{:.2$} {:.2$}\n",
                                     peak.min as f32 / 0xFFFF as f32,
                                     peak.max as f32 / 0xFFFF as f32,
                                     args.precision),
    }
}
