use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Fifo {
    data: Vec<f32>,
    read: usize,
    write: usize,
    len: usize,
}

impl Fifo {
    pub fn with_capacity(frames: usize) -> Result<Self> {
        if frames == 0 {
            return Err(Error::InvalidArgument);
        }

        Ok(Self {
            data: vec![0.0; frames],
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
        let count = input.len().min(self.available_to_write());
        for sample in input.iter().take(count) {
            self.data[self.write] = *sample;
            self.write = (self.write + 1) % self.data.len();
        }
        self.len += count;
        count
    }

    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let count = output.len().min(self.available_to_read());
        for slot in output.iter_mut().take(count) {
            *slot = self.data[self.read];
            self.read = (self.read + 1) % self.data.len();
        }
        self.len -= count;
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_truncates_writes_and_preserves_order_across_wrap() {
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
