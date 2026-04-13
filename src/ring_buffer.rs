/// Fixed-capacity circular buffer for time-series chart data.
///
/// Stores the most recent `capacity` samples. Oldest samples are
/// silently dropped when the buffer is full. Iterates in insertion
/// order (oldest to newest).
pub struct RingBuffer {
    buf: Vec<f64>,
    head: usize,
    len: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0.0; capacity],
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: f64) {
        self.buf[self.head] = value;
        self.head = (self.head + 1) % self.buf.len();
        if self.len < self.buf.len() {
            self.len += 1;
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn latest(&self) -> Option<f64> {
        if self.len == 0 {
            return None;
        }
        let index = if self.head == 0 {
            self.buf.len() - 1
        } else {
            self.head - 1
        };
        Some(self.buf[index])
    }

    /// Produces (x, y) pairs for ratatui Chart data.
    pub fn as_chart_data(&self, out: &mut Vec<(f64, f64)>) {
        out.clear();
        out.extend(self.iter().enumerate().map(|(idx, val)| (idx as f64, val)));
    }

    fn iter(&self) -> RingBufferIter<'_> {
        RingBufferIter {
            buf: &self.buf,
            start: if self.len < self.buf.len() {
                0
            } else {
                self.head
            },
            remaining: self.len,
            capacity: self.buf.len(),
        }
    }
}

struct RingBufferIter<'a> {
    buf: &'a [f64],
    start: usize,
    remaining: usize,
    capacity: usize,
}

impl Iterator for RingBufferIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let value = self.buf[self.start];
        self.start = (self.start + 1) % self.capacity;
        self.remaining -= 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for RingBufferIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_iterate() {
        let mut ring = RingBuffer::new(3);
        ring.push(1.0);
        ring.push(2.0);
        ring.push(3.0);
        ring.push(4.0);

        let values: Vec<f64> = ring.iter().collect();
        assert_eq!(values, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_latest() {
        let mut ring = RingBuffer::new(3);
        ring.push(10.0);
        ring.push(20.0);
        assert_eq!(ring.latest(), Some(20.0));
    }

    #[test]
    fn test_as_chart_data() {
        let mut ring = RingBuffer::new(3);
        ring.push(10.0);
        ring.push(20.0);
        ring.push(30.0);
        ring.push(40.0);

        let mut data = Vec::new();
        ring.as_chart_data(&mut data);
        assert_eq!(data, vec![(0.0, 20.0), (1.0, 30.0), (2.0, 40.0)]);
    }
}
