use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    // List all input devices
    println!("Available input devices:");
    for device in host.input_devices()? {
        println!("  {}", device.name()?);
    }

    // Try to pick the default input device (often "Stereo Mix" or "Speakers (loopback)")
    let device = host
        .default_input_device()
        .expect("No default input device available");

    println!("Capturing audio from: {}", device.name()?);

    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0;
    let config = config.into();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let samples_clone = Arc::clone(&samples);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            let mut buffer = samples_clone.lock().unwrap();
            buffer.extend_from_slice(data);
        },
        move |err| {
            eprintln!("Stream error: {:?}", err);
        },
        None,
    )?;

    println!("Recording for 10 seconds...");
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(10));

    drop(stream);

    let samples = samples.lock().unwrap();
    save_wav("output.wav", &samples, sample_rate)?;
    println!("Saved recording to output.wav");

    Ok(())
}

fn save_wav(path: &str, samples: &[f32], sample_rate: u32) -> Result<(), anyhow::Error> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut writer = hound::WavWriter::new(writer, spec)?;

    for &sample in samples {
        let val = (sample * i16::MAX as f32) as i16;
        writer.write_sample(val)?;
    }

    writer.finalize()?;
    Ok(())
}
