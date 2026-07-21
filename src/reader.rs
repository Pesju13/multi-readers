use crate::segment::Segment;

/// Presents a sequence of readers as one continuous reader.
///
/// Readers are consumed in insertion order. Empty readers are skipped. When
/// the inner readers also implement [`std::io::Seek`], this type exposes a
/// single logical seek position across the concatenated stream.
#[derive(Debug)]
pub struct MultiReader<R> {
    pub(crate) segments: Vec<Segment<R>>,
    pub(crate) current: usize,
    pub(crate) position: u64,
    pub(crate) total_len: Option<u64>,
}

impl<R> MultiReader<R> {
    /// Creates a reader from any collection or iterator of readers.
    pub fn new(readers: impl IntoIterator<Item = R>) -> Self {
        Self {
            segments: readers.into_iter().map(Segment::new).collect(),
            current: 0,
            position: 0,
            total_len: None,
        }
    }

    /// Creates a reader from fallible reader values.
    ///
    /// Construction stops at the first error instead of silently discarding
    /// failed values.
    ///
    /// ```rust
    /// use std::io::Cursor;
    /// use multi_readers::MultiReader;
    /// # use std::io;
    /// let values: [io::Result<Cursor<&[u8]>>; 2] = [
    ///     Ok(Cursor::new(b"hello")),
    ///     Ok(Cursor::new(b"world")),
    /// ];
    /// let reader = MultiReader::try_new(values)?;
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn try_new<E>(readers: impl IntoIterator<Item = Result<R, E>>) -> Result<Self, E> {
        readers
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map(Self::new)
    }

    /// Returns the number of inner readers.
    pub fn reader_count(&self) -> usize {
        self.segments.len()
    }

    /// Returns `true` when no readers were supplied.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the logical byte position maintained by this adapter.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Iterates over the inner readers without exposing adapter state.
    pub fn readers(&self) -> impl ExactSizeIterator<Item = &R> + DoubleEndedIterator {
        self.segments.iter().map(Segment::get_ref)
    }

    /// Consumes the adapter and returns its inner readers.
    pub fn into_readers(self) -> Vec<R> {
        self.segments.into_iter().map(Segment::into_inner).collect()
    }

    pub(crate) fn advance_position(&mut self, amount: usize) {
        self.position = self.position.saturating_add(amount as u64);
    }
}

impl<R> Default for MultiReader<R> {
    fn default() -> Self {
        Self::new(std::iter::empty())
    }
}

impl<R> FromIterator<R> for MultiReader<R> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl<R> From<Vec<R>> for MultiReader<R> {
    fn from(readers: Vec<R>) -> Self {
        Self::new(readers)
    }
}
