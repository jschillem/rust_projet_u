use projet_u::{Wav, sample::Samples};

fn main() {
    const DURATION_SECS: u32 = 5;
    const SAMPLE_RATE: u32 = 44100;
    const NUM_CHANNELS: u16 = 1;
    const NUM_SAMPLES: usize = (DURATION_SECS * SAMPLE_RATE) as usize;

    let samples: Vec<i16> = (0..NUM_SAMPLES)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let frequency = 440.0; // A4 note
            let amplitude = i16::MAX as f32 * 0.5; // 50% volume
            (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16
        })
        .collect();

    let audio_samples = Samples::from_vec(samples, NUM_CHANNELS, SAMPLE_RATE);

    let wav = Wav::from_samples(audio_samples);

    let mut file = std::fs::File::create("output.wav").expect("Failed to create file");

    wav.write_to(&mut file).expect("Failed to write WAV data");
}
