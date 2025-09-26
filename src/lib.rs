use std::io::Write;

use sample::Sample;

use crate::sample::Samples;

pub mod sample;

type Fourcc = [u8; 4];

struct RiffHeader {
    id: Fourcc,
    size: u32,
    format: Fourcc,
}

#[derive(Clone, Copy)]
enum AudioFormat {
    Pcm = 1,   // PCM format
    Float = 3, // IEEE float format
}

struct FmtChunk {
    id: Fourcc,
    size: u32,
    audio_format: AudioFormat,
    num_channels: u16,
    samples_per_sec: u32,
    bytes_per_sec: u32,
    block_align: u16,
    bits_per_sample: u16,
}

struct DataChunk {
    id: Fourcc,
    size: u32,
}

/// Represents audio data in WAV format.
pub struct Wav<T: Sample> {
    riff_header: RiffHeader,
    fmt_chunk: FmtChunk,
    data_chunk: DataChunk,
    data: Samples<T>,
}

impl<T: Sample> Wav<T> {
    pub fn from_samples(samples: Samples<T>) -> Self {
        let num_channels = samples.num_channels();
        let samples_per_sec = samples.sample_rate();

        let bits_per_sample = T::BYTES as u16 * 8;
        let byte_rate = samples_per_sec * num_channels as u32 * T::BYTES as u32;
        let block_align = num_channels * T::BYTES as u16;
        let data_size = samples.len() as u32 * T::BYTES as u32;

        Self {
            riff_header: RiffHeader {
                id: *b"RIFF",
                size: 36 + data_size,
                format: *b"WAVE",
            },
            fmt_chunk: FmtChunk {
                id: *b"fmt ",
                size: 16,
                audio_format: if T::IS_FLOAT {
                    AudioFormat::Float
                } else {
                    AudioFormat::Pcm
                },
                num_channels,
                samples_per_sec,
                bytes_per_sec: byte_rate,
                block_align,
                bits_per_sample,
            },
            data_chunk: DataChunk {
                id: *b"data",
                size: data_size,
            },
            data: samples,
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Write RIFF header
        writer.write_all(&self.riff_header.id)?;
        writer.write_all(&self.riff_header.size.to_le_bytes())?;
        writer.write_all(&self.riff_header.format)?;

        // Write fmt chunk
        writer.write_all(&self.fmt_chunk.id)?;
        writer.write_all(&self.fmt_chunk.size.to_le_bytes())?;
        writer.write_all(&(self.fmt_chunk.audio_format as u16).to_le_bytes())?;
        writer.write_all(&self.fmt_chunk.num_channels.to_le_bytes())?;
        writer.write_all(&self.fmt_chunk.samples_per_sec.to_le_bytes())?;
        writer.write_all(&self.fmt_chunk.bytes_per_sec.to_le_bytes())?;
        writer.write_all(&self.fmt_chunk.block_align.to_le_bytes())?;
        writer.write_all(&self.fmt_chunk.bits_per_sample.to_le_bytes())?;

        // Write data chunk
        writer.write_all(&self.data_chunk.id)?;
        writer.write_all(&self.data_chunk.size.to_le_bytes())?;

        for sample in self.data.iter() {
            writer.write_all(sample.to_bytes().as_ref())?;
        }

        // determine if we need to write a padding byte
        if !self.data_chunk.size.is_multiple_of(2) {
            writer.write_all(&[0u8])?;
        }

        Ok(())
    }
}
