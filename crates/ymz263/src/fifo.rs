#[derive(Clone)]
pub(crate) struct FixedFifo<const N: usize> {
    bytes: [u8; N],
    head: usize,
    len: usize,
}

impl<const N: usize> FixedFifo<N> {
    pub(crate) const fn new() -> Self {
        Self {
            bytes: [0; N],
            head: 0,
            len: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn is_full(&self) -> bool {
        self.len == N
    }

    pub(crate) fn push(&mut self, value: u8) -> Result<(), u8> {
        if self.is_full() {
            return Err(value);
        }
        let tail = (self.head + self.len) % N;
        self.bytes[tail] = value;
        self.len += 1;
        Ok(())
    }

    pub(crate) fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let value = self.bytes[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(value)
    }

    pub(crate) fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }
}
