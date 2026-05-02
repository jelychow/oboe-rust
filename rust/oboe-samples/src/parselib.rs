use oboe_core::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WavEncoding {
    Pcm,
    IeeeFloat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WavData {
    pub channel_count: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub encoding: WavEncoding,
    pub frames: Vec<f32>,
}

impl WavData {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(Error::InvalidArgument);
        }

        let mut cursor = 12;
        let mut fmt = None;
        let mut data_range = None;

        while cursor + 8 <= bytes.len() {
            let id = &bytes[cursor..cursor + 4];
            let size = read_u32(bytes, cursor + 4)? as usize;
            let start = cursor + 8;
            let end = start.checked_add(size).ok_or(Error::InvalidArgument)?;
            if end > bytes.len() {
                return Err(Error::InvalidArgument);
            }

            match id {
                b"fmt " => fmt = Some(parse_fmt_chunk(&bytes[start..end])?),
                b"data" => data_range = Some(start..end),
                _ => {}
            }

            cursor = end + (size % 2);
        }

        let fmt = fmt.ok_or(Error::InvalidArgument)?;
        let data = data_range.ok_or(Error::InvalidArgument)?;
        let samples = parse_samples(&bytes[data], fmt.bits_per_sample, fmt.encoding)?;

        Ok(Self {
            channel_count: fmt.channel_count,
            sample_rate: fmt.sample_rate,
            bits_per_sample: fmt.bits_per_sample,
            encoding: fmt.encoding,
            frames: samples,
        })
    }
}

#[derive(Clone, Copy)]
struct FmtChunk {
    channel_count: u16,
    sample_rate: u32,
    bits_per_sample: u16,
    encoding: WavEncoding,
}

fn parse_fmt_chunk(chunk: &[u8]) -> Result<FmtChunk> {
    if chunk.len() < 16 {
        return Err(Error::InvalidArgument);
    }

    let format_code = read_u16(chunk, 0)?;
    let encoding = match format_code {
        1 => WavEncoding::Pcm,
        3 => WavEncoding::IeeeFloat,
        _ => return Err(Error::InvalidArgument),
    };

    let channel_count = read_u16(chunk, 2)?;
    if channel_count == 0 {
        return Err(Error::InvalidArgument);
    }

    Ok(FmtChunk {
        channel_count,
        sample_rate: read_u32(chunk, 4)?,
        bits_per_sample: read_u16(chunk, 14)?,
        encoding,
    })
}

fn parse_samples(bytes: &[u8], bits_per_sample: u16, encoding: WavEncoding) -> Result<Vec<f32>> {
    match (encoding, bits_per_sample) {
        (WavEncoding::Pcm, 8) => Ok(bytes
            .iter()
            .map(|sample| (*sample as f32 - 128.0) / 128.0)
            .collect()),
        (WavEncoding::Pcm, 16) => parse_chunks(bytes, 2, |chunk| {
            i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32_768.0
        }),
        (WavEncoding::Pcm, 24) => parse_chunks(bytes, 3, |chunk| {
            let value =
                ((chunk[0] as i32) << 8) | ((chunk[1] as i32) << 16) | ((chunk[2] as i32) << 24);
            value as f32 / 2_147_483_648.0
        }),
        (WavEncoding::Pcm, 32) => parse_chunks(bytes, 4, |chunk| {
            i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32 / 2_147_483_648.0
        }),
        (WavEncoding::IeeeFloat, 32) => parse_chunks(bytes, 4, |chunk| {
            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        }),
        _ => Err(Error::InvalidArgument),
    }
}

fn parse_chunks(
    bytes: &[u8],
    chunk_size: usize,
    convert: impl Fn(&[u8]) -> f32,
) -> Result<Vec<f32>> {
    if bytes.len() % chunk_size != 0 {
        return Err(Error::InvalidArgument);
    }
    Ok(bytes.chunks_exact(chunk_size).map(convert).collect())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let end = offset.checked_add(2).ok_or(Error::InvalidArgument)?;
    let slice = bytes.get(offset..end).ok_or(Error::InvalidArgument)?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).ok_or(Error::InvalidArgument)?;
    let slice = bytes.get(offset..end).ok_or(Error::InvalidArgument)?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

pub fn write_test_wav_i16(channel_count: u16, sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let data_size = samples.len() as u32 * 2;
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * channel_count as u32 * 2).to_le_bytes());
    bytes.extend_from_slice(&(channel_count * 2).to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pcm16_wav_to_float_samples() {
        let wav = write_test_wav_i16(1, 48_000, &[i16::MIN, 0, i16::MAX]);
        let parsed = WavData::parse(&wav).unwrap();
        assert_eq!(parsed.bits_per_sample, 16);
        assert_eq!(parsed.frames[0], -1.0);
        assert_eq!(parsed.frames[1], 0.0);
        assert!((parsed.frames[2] - (32_767.0 / 32_768.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_missing_riff_header() {
        assert_eq!(WavData::parse(b"not a wav"), Err(Error::InvalidArgument));
    }
}
