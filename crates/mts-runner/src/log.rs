use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub stream: Stream,
    pub text: String,
}

/// The last `capacity` lines; older ones are dropped and counted.
#[derive(Debug, Clone)]
pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    capacity: usize,
    dropped: u64,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            capacity,
            dropped: 0,
        }
    }

    pub fn push(&mut self, line: LogLine) {
        if self.lines.len() == self.capacity {
            self.lines.pop_front();
            self.dropped += 1;
        }
        self.lines.push_back(line);
    }

    pub fn lines(&self) -> Vec<LogLine> {
        self.lines.iter().cloned().collect()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_newest_lines() {
        let mut b = LogBuffer::new(3);
        for i in 0..5 {
            b.push(LogLine {
                stream: Stream::Stdout,
                text: i.to_string(),
            });
        }
        let texts: Vec<_> = b.lines().into_iter().map(|l| l.text).collect();
        assert_eq!(texts, ["2", "3", "4"]);
        assert_eq!(b.dropped(), 2);
    }
}
