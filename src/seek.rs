use std::io::{self, Seek, SeekFrom};

use crate::MultiReader;

impl<R: Seek> MultiReader<R> {
    /// Returns the combined length of all inner readers.
    ///
    /// Lengths are cached because determining them requires seeking each inner
    /// reader to its end and restoring its position. Inner stream lengths must
    /// therefore remain stable after this method or [`Seek::seek`] is used.
    pub fn total_len(&mut self) -> io::Result<u64> {
        if let Some(len) = self.total_len {
            return Ok(len);
        }

        let mut total = 0_u64;
        for segment in &mut self.segments {
            total = total.checked_add(segment.len()?).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "combined reader length exceeds u64::MAX",
                )
            })?;
        }

        self.total_len = Some(total);
        Ok(total)
    }

    fn seek_absolute(&mut self, target: u64) -> io::Result<u64> {
        let mut offset = 0_u64;
        let mut destination = None;

        for (index, segment) in self.segments.iter_mut().enumerate() {
            let end = offset.checked_add(segment.len()?).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "combined reader length exceeds u64::MAX",
                )
            })?;

            if target < end {
                destination = Some((index, offset));
                break;
            }
            offset = end;
        }

        let Some((index, segment_start)) = destination else {
            self.current = self.segments.len();
            self.position = target;
            return Ok(target);
        };

        let local_target = target - segment_start;
        let local_position = self.segments[index].seek(SeekFrom::Start(local_target))?;
        let position = segment_start.checked_add(local_position).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "seek position exceeds u64::MAX")
        })?;

        for segment in &mut self.segments[index + 1..] {
            segment.seek(SeekFrom::Start(0))?;
        }

        self.current = index;
        self.position = position;
        Ok(position)
    }
}

impl<R: Seek> Seek for MultiReader<R> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let target = match from {
            SeekFrom::Start(position) => Some(position),
            SeekFrom::Current(offset) => self.position.checked_add_signed(offset),
            SeekFrom::End(offset) => self.total_len()?.checked_add_signed(offset),
        }
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot seek before the start of the combined reader",
            )
        })?;

        self.seek_absolute(target)
    }
}
