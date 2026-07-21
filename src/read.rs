use std::io::Read;

use crate::MultiReader;

impl<R: Read> Read for MultiReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            let Some(reader) = self.segments.get_mut(self.current) else {
                return Ok(0);
            };

            let read = reader.read(buf)?;
            if read == 0 {
                self.current += 1;
                continue;
            }

            self.advance_position(read);
            return Ok(read);
        }
    }
}
