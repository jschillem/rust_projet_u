use projet_u::{Wav, sample::Samples};

fn main() {
    const DURATION_SECS: u32 = 32;
    const SAMPLE_RATE: u32 = 8000;
    const NUM_CHANNELS: u16 = 1;
    const NUM_SAMPLES: usize = (DURATION_SECS * SAMPLE_RATE) as usize;

    let samples: Vec<u8> = (0..NUM_SAMPLES)
        .map(|t| (t * 5 & t >> 7 | t * 5 & t >> 8) as u8)
        .collect();

    let audio_samples = Samples::from_vec(samples, NUM_CHANNELS, SAMPLE_RATE);

    let wav = Wav::from_samples(audio_samples);

    let mut file = std::fs::File::create("output.wav").expect("Failed to create file");

    wav.write_to(&mut file).expect("Failed to write WAV data");
}
