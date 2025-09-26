use i24::i24;
use std::borrow::Cow;

/// Trait representing a sample type in a WAV file.
pub trait Sample: Copy + Clone + Sized + 'static {
    const BYTES: u8;
    const BITS: u16 = Self::BYTES as u16 * 8;
    const IS_FLOAT: bool = false;

    /// A byte array type that can hold the sample's byte representation.
    type ByteArray: AsRef<[u8]> + AsMut<[u8]>;

    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Self::ByteArray;
}

impl Sample for u8 {
    const BYTES: u8 = 1;
    type ByteArray = [u8; 1];

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }

    fn to_bytes(&self) -> Self::ByteArray {
        [*self]
    }
}

impl Sample for i16 {
    const BYTES: u8 = 2;

    type ByteArray = [u8; 2];

    fn from_bytes(bytes: &[u8]) -> Self {
        i16::from_le_bytes([bytes[0], bytes[1]])
    }

    fn to_bytes(&self) -> Self::ByteArray {
        self.to_le_bytes()
    }
}

impl Sample for i24 {
    const BYTES: u8 = 3;

    type ByteArray = [u8; 3];

    fn from_bytes(bytes: &[u8]) -> Self {
        i24::from_le_bytes([bytes[0], bytes[1], bytes[2]])
    }

    fn to_bytes(&self) -> Self::ByteArray {
        self.to_le_bytes()
    }
}

impl Sample for f32 {
    const BYTES: u8 = 4;
    const IS_FLOAT: bool = true;

    type ByteArray = [u8; 4];

    fn from_bytes(bytes: &[u8]) -> Self {
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_bytes(&self) -> Self::ByteArray {
        self.to_le_bytes()
    }
}

pub struct Samples<T: Sample> {
    data: Cow<'static, [T]>,
    num_channels: u16,
    samples_per_sec: u32,
}

impl<T: Sample> Samples<T> {
    pub const fn from_vec(vec: Vec<T>, num_channels: u16, samples_per_sec: u32) -> Self {
        Self {
            data: Cow::Owned(vec),
            num_channels,
            samples_per_sec,
        }
    }

    pub const fn borrowed(data: &'static [T], num_channels: u16, samples_per_sec: u32) -> Self {
        Self {
            data: Cow::Borrowed(data),
            num_channels,
            samples_per_sec,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn frame_count(&self) -> usize {
        self.len() / self.num_channels as usize
    }

    pub fn duration_secs(&self) -> f32 {
        self.frame_count() as f32 / self.samples_per_sec as f32
    }

    pub fn frames(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks_exact(self.num_channels as usize)
    }

    pub fn channel(&self, channel: u16) -> impl Iterator<Item = &T> {
        debug_assert!(channel < self.num_channels);

        self.data
            .iter()
            .skip(channel as usize)
            .step_by(self.num_channels as usize)
    }

    pub fn num_channels(&self) -> u16 {
        self.num_channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.samples_per_sec
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.to_mut().iter_mut()
    }

    pub fn try_iter_mut(&mut self) -> Option<std::slice::IterMut<'_, T>> {
        match &mut self.data {
            Cow::Owned(vec) => Some(vec.iter_mut()),
            Cow::Borrowed(_) => None,
        }
    }

    pub fn make_owned(&mut self) {
        if matches!(self.data, Cow::Borrowed(_)) {
            self.data = Cow::Owned(self.data.to_vec());
        }
    }
}

impl<T: Sample> std::ops::Index<usize> for Samples<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Sample> std::ops::Index<std::ops::Range<usize>> for Samples<T> {
    type Output = [T];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Sample> std::ops::Index<std::ops::RangeFull> for Samples<T> {
    type Output = [T];

    fn index(&self, _index: std::ops::RangeFull) -> &Self::Output {
        &self.data
    }
}

impl<T: Sample> std::ops::Index<std::ops::RangeFrom<usize>> for Samples<T> {
    type Output = [T];

    fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
        &self.data[index]
    }
}
