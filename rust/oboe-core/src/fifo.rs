use crate::error::{Error, Result};

/// Ring buffer for scalar `f32` samples.
///
/// Frame-aware buffering should layer channel-count handling above this FIFO.
#[derive(Debug)]
pub struct Fifo {
    data: Vec<f32>,
    read: usize,
    write: usize,
    len: usize,
}

impl Fifo {
    pub fn with_capacity(sample_capacity: usize) -> Result<Self> {
        if sample_capacity == 0 {
            return Err(Error::InvalidArgument);
        }

        Ok(Self {
            data: vec![0.0; sample_capacity],
            read: 0,
            write: 0,
            len: 0,
        })
    }

    pub fn available_to_read(&self) -> usize {
        self.len
    }

    pub fn available_to_write(&self) -> usize {
        self.data.len() - self.len
    }

    pub fn write(&mut self, input: &[f32]) -> usize {
        let sample_count = input.len().min(self.available_to_write());
        for sample in input.iter().take(sample_count) {
            self.data[self.write] = *sample;
            self.write = (self.write + 1) % self.data.len();
        }
        self.len += sample_count;
        sample_count
    }

    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let sample_count = output.len().min(self.available_to_read());
        for slot in output.iter_mut().take(sample_count) {
            *slot = self.data[self.read];
            self.read = (self.read + 1) % self.data.len();
        }
        self.len -= sample_count;
        sample_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_sample_capacity() {
        assert!(matches!(
            Fifo::with_capacity(0),
            Err(Error::InvalidArgument)
        ));
    }

    #[test]
    fn empty_read_is_a_no_op() {
        let mut fifo = Fifo::with_capacity(3).unwrap();
        let mut output = [9.0, 8.0];
        assert_eq!(fifo.read(&mut output), 0);
        assert_eq!(output, [9.0, 8.0]);
        assert_eq!(fifo.available_to_read(), 0);
        assert_eq!(fifo.available_to_write(), 3);
    }

    #[test]
    fn full_write_truncates_and_additional_write_is_no_op() {
        let mut fifo = Fifo::with_capacity(3).unwrap();
        assert_eq!(fifo.write(&[1.0, 2.0, 3.0, 4.0]), 3);
        assert_eq!(fifo.available_to_read(), 3);
        assert_eq!(fifo.available_to_write(), 0);
        assert_eq!(fifo.write(&[5.0]), 0);

        let mut output = [0.0; 3];
        assert_eq!(fifo.read(&mut output), 3);
        assert_eq!(output, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn sample_fifo_truncates_writes_and_preserves_order_across_wrap() {
        let mut fifo = Fifo::with_capacity(3).unwrap();
        assert_eq!(fifo.write(&[1.0, 2.0, 3.0, 4.0]), 3);
        let mut first = [0.0; 2];
        assert_eq!(fifo.read(&mut first), 2);
        assert_eq!(first, [1.0, 2.0]);
        assert_eq!(fifo.write(&[4.0, 5.0]), 2);
        let mut second = [0.0; 3];
        assert_eq!(fifo.read(&mut second), 3);
        assert_eq!(second, [3.0, 4.0, 5.0]);
    }
}
