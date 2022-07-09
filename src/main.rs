use hound;
use std::i16;
use clap::Parser;

const PI : f64 = 3.14159;
const SAMPLES_PER_SEC : f64 = 44100.;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    input_wav: String,
    #[clap(short, long, value_parser)]
    output_wav: String,
    #[clap(short, long, value_parser)]
    transform: String,
    #[clap(short, long, value_parser)]
    amplitude: f64,
    #[clap(short, long, value_parser)]
    wavelength_sec: f64 
}

type ActionFunc = fn( p: &Cli , i: &Vec<i16> ) ->  Vec<i16>;

struct TransformAction {
    transform_name: String,
    transform_func: ActionFunc
}


// FIXME: can we pass in_samples as a reference, and just make copy to start with?

fn xform_null( args : &Cli, in_samples: &Vec<i16> ) -> Vec<i16> {
    let mut out_samples = Vec::with_capacity(in_samples.len());
    let a = args.amplitude;
    for x in in_samples {
        let next_sample = (a * (*x as f64)) as i16 ;
        out_samples.push( next_sample );
    }
    out_samples
}

fn xform_tremolo( args : &Cli, in_samples: &Vec<i16> ) -> Vec<i16> {
    let mut out_samples = Vec::with_capacity(in_samples.len());
    let a = args.amplitude;
    let sec_per_radian = PI / (args.wavelength_sec * SAMPLES_PER_SEC);

    for (i, x) in in_samples.iter().enumerate() {
        let x2 = *x as f64;
        let radians : f64 = (i as f64) * sec_per_radian;
        // the "wavelength" here is really PI radians, not TWO_PI.  
        // at PI radians, the cos function goes negative and
        // then with amplitude 0.5 at 3PI/2 radians the two terms will cancel!
        // so instead we use abs(sin(radians)) so that the two terms
        // in next_sample reinforce each other,
        // FIXME: should we use cos^2 to get a non-negative function?
        let next_sample = (a * x2 * radians.cos().abs()) + ((1.0-a) * x2);
        out_samples.push(next_sample as i16);
    }                         
    out_samples
}

fn xform_delay( args: &Cli, in_samples: &Vec<i16> ) -> Vec<i16> {
    let mut out_samples = Vec::with_capacity(in_samples.len());
    let a = args.amplitude;
    let delay_in_samples = (args.wavelength_sec * SAMPLES_PER_SEC) as usize;

    for (i, x) in in_samples.iter().enumerate() {
        let x2 = *x as f64;
        let next_sample = if i > delay_in_samples {
            let i_minus_delta = i - delay_in_samples;
           ((1.0 - a)*x2) + (a * (out_samples[i_minus_delta] as f64))
        } else {
            x2
        };
        out_samples.push(next_sample as i16);
    }                         
    out_samples
}

fn transform( args : &Cli, in_samples: &Vec<i16> ) -> Vec<i16> {
    let transforms : [TransformAction; 3] = [ 
        TransformAction{ transform_name:"tremolo".to_string(), transform_func:xform_tremolo},
        TransformAction{ transform_name:"delay".to_string(),   transform_func:xform_delay},
        TransformAction{ transform_name:"none".to_string(),    transform_func:xform_null}
        ];
    let xfrm = args.transform.to_string();
    let mut out_samples : Vec<i16> = Vec::with_capacity(0);
    for t in transforms {
        if t.transform_name == xfrm {
            let f = t.transform_func;
            out_samples = f(args, in_samples);
        }
    };
    out_samples
}

fn main() {
    let args = Cli::parse();
    println!("input wav file: {}", args.input_wav);
    println!("output wav file: {}", args.output_wav);
    println!("transform: {}", args.transform);
    println!("amplitude: {}", args.amplitude);
    println!("wavelength (sec): {}", args.wavelength_sec);
    
    let input_wav = args.input_wav.clone();
    let mut reader = hound::WavReader::open(input_wav).unwrap();
    let read_spec = reader.spec();
    let samples : Vec<i16> = reader.samples::<i16>()
                                   .map(|r| r.unwrap())
                                   .collect();
    let out_spec = hound::WavSpec {
        bits_per_sample: 16,
        channels: read_spec.channels,
        sample_format: hound::SampleFormat::Int,
        sample_rate: read_spec.sample_rate,
    };
    let output_wav = args.output_wav.clone();
    let mut writer = hound::WavWriter::create(output_wav, out_spec).unwrap();

    let mut writer_i16 = writer.get_i16_writer(samples.len() as u32);

    let out_samples = transform( &args, &samples );
    if out_samples.len() == 0 {
        panic!("ERROR: unrecognized transform {} not performed", args.transform);
    }

    // FIXME: why do we need a loop here?
    for o in out_samples {
        writer_i16.write_sample(o);
    }
    // FIXME: what happens if you leave this out?
    writer_i16.flush().unwrap();
}
