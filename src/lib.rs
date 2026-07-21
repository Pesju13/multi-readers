#![doc = include_str!("../README.md")]

mod read;
mod reader;
mod seek;
mod segment;

#[cfg(feature = "tokio")]
mod tokio_read;

pub use reader::MultiReader;

#[deprecated(since = "0.7.0", note = "use `MultiReader` instead")]
pub type MultiReaders<R> = MultiReader<R>;
