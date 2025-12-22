//! The fastest semantic text chunking library — up to 1TB/s chunking throughput.
//!
//! # Example
//!
//! ```
//! use memchunk::chunk;
//!
//! let text = b"Hello world. How are you? I'm fine.\nThanks for asking.";
//!
//! // With defaults (4KB chunks, split at \n . ?)
//! let chunks: Vec<&[u8]> = chunk(text).collect();
//!
//! // With custom size and delimiters
//! let chunks: Vec<&[u8]> = chunk(text).size(1024).delimiters(b"\n.?!").collect();
//! ```

/// Default chunk target size (4KB).
pub const DEFAULT_TARGET_SIZE: usize = 4096;

/// Default delimiters: newline, period, question mark.
pub const DEFAULT_DELIMITERS: &[u8] = b"\n.?";

/// Chunk text at delimiter boundaries.
///
/// Returns a builder that can be configured with `.size()` and `.delimiters()`,
/// or used directly as an iterator with defaults (4KB chunks, `\n.?` delimiters).
///
/// - For 1-3 delimiters: uses SIMD-accelerated memchr
/// - For 4+ delimiters: uses lookup table
///
/// # Example
///
/// ```
/// use memchunk::chunk;
///
/// let text = b"First sentence. Second sentence. Third sentence.";
///
/// // With defaults
/// let chunks: Vec<_> = chunk(text).collect();
///
/// // With custom size
/// let chunks: Vec<_> = chunk(text).size(1024).collect();
///
/// // With custom delimiters
/// let chunks: Vec<_> = chunk(text).delimiters(b"\n.?!").collect();
///
/// // With both
/// let chunks: Vec<_> = chunk(text).size(8192).delimiters(b"\n").collect();
/// ```
pub fn chunk(text: &[u8]) -> Chunker<'_> {
    Chunker::new(text)
}

/// Chunker splits text at delimiter boundaries.
///
/// Created via [`chunk()`], can be configured with `.size()` and `.delimiters()`.
pub struct Chunker<'a> {
    text: &'a [u8],
    target_size: usize,
    delimiters: &'a [u8],
    pos: usize,
    table: Option<[bool; 256]>,
    initialized: bool,
}

impl<'a> Chunker<'a> {
    fn new(text: &'a [u8]) -> Self {
        Self {
            text,
            target_size: DEFAULT_TARGET_SIZE,
            delimiters: DEFAULT_DELIMITERS,
            pos: 0,
            table: None,
            initialized: false,
        }
    }

    /// Set the target chunk size in bytes.
    pub fn size(mut self, size: usize) -> Self {
        self.target_size = size;
        self
    }

    /// Set the delimiters to split on.
    pub fn delimiters(mut self, delimiters: &'a [u8]) -> Self {
        self.delimiters = delimiters;
        self
    }

    /// Initialize lookup table if needed (called on first iteration).
    fn init(&mut self) {
        if !self.initialized {
            if self.delimiters.len() > 3 {
                let mut t = [false; 256];
                for &b in self.delimiters {
                    t[b as usize] = true;
                }
                self.table = Some(t);
            }
            self.initialized = true;
        }
    }

    /// Find last delimiter in window using appropriate method.
    #[inline]
    fn find_last_delimiter(&self, window: &[u8]) -> Option<usize> {
        if let Some(ref table) = self.table {
            // Lookup table for 4+ delimiters
            window.iter().rposition(|&b| table[b as usize])
        } else {
            // SIMD-accelerated memchr for 1-3 delimiters
            match self.delimiters.len() {
                1 => memchr::memrchr(self.delimiters[0], window),
                2 => memchr::memrchr2(self.delimiters[0], self.delimiters[1], window),
                3 => memchr::memrchr3(
                    self.delimiters[0],
                    self.delimiters[1],
                    self.delimiters[2],
                    window,
                ),
                0 => None,
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Iterator for Chunker<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.init();

        if self.pos >= self.text.len() {
            return None;
        }

        let remaining = self.text.len() - self.pos;

        // Last chunk - return remainder
        if remaining <= self.target_size {
            let chunk = &self.text[self.pos..];
            self.pos = self.text.len();
            return Some(chunk);
        }

        let end = self.pos + self.target_size;
        let window = &self.text[self.pos..end];

        // Find last delimiter in window
        let split_at = match self.find_last_delimiter(window) {
            Some(pos) => self.pos + pos + 1, // Include the delimiter
            None => end,                      // No delimiter, hard split at target
        };

        let chunk = &self.text[self.pos..split_at];
        self.pos = split_at;
        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let text = b"Hello. World. Test.";
        let chunks: Vec<_> = chunk(text).size(10).delimiters(b".").collect();
        // "Hello." (6) + " World." (7) + " Test." (6) = 19
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], b"Hello.");
        assert_eq!(chunks[1], b" World.");
        assert_eq!(chunks[2], b" Test.");
    }

    #[test]
    fn test_newline_delimiter() {
        let text = b"Line one\nLine two\nLine three";
        let chunks: Vec<_> = chunk(text).size(15).delimiters(b"\n").collect();
        assert_eq!(chunks[0], b"Line one\n");
        assert_eq!(chunks[1], b"Line two\n");
        assert_eq!(chunks[2], b"Line three");
    }

    #[test]
    fn test_multiple_delimiters() {
        let text = b"Hello? World. Yes!";
        let chunks: Vec<_> = chunk(text).size(10).delimiters(b".?!").collect();
        assert_eq!(chunks[0], b"Hello?");
    }

    #[test]
    fn test_four_delimiters_uses_table() {
        let text = b"A. B? C! D; E";
        let chunks: Vec<_> = chunk(text).size(5).delimiters(b".?!;").collect();
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_no_delimiter_hard_split() {
        let text = b"abcdefghij";
        let chunks: Vec<_> = chunk(text).size(5).delimiters(b".").collect();
        assert_eq!(chunks[0], b"abcde");
        assert_eq!(chunks[1], b"fghij");
    }

    #[test]
    fn test_empty_text() {
        let text = b"";
        let chunks: Vec<_> = chunk(text).size(10).delimiters(b".").collect();
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_text_smaller_than_target() {
        let text = b"Small";
        let chunks: Vec<_> = chunk(text).size(100).delimiters(b".").collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], b"Small");
    }

    #[test]
    fn test_total_bytes_preserved() {
        let text = b"The quick brown fox jumps over the lazy dog. How vexingly quick!";
        let chunks: Vec<_> = chunk(text).size(20).delimiters(b"\n.?!").collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_defaults() {
        let text = b"Hello world. This is a test.";
        // Should work with just defaults
        let chunks: Vec<_> = chunk(text).collect();
        assert!(!chunks.is_empty());
    }
}
