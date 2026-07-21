use std::io::{Cursor, Read};

use multi_readers::MultiReader;

fn main() -> std::io::Result<()> {
    let mut readers = MultiReader::new([Cursor::new("Hello, "), Cursor::new("World!")]);
    let mut hello_world = String::new();
    readers.read_to_string(&mut hello_world)?;
    assert_eq!(hello_world.as_str(), "Hello, World!");

    let readers: Vec<Box<dyn Read>> = vec![
        Box::new(Cursor::new("Hello, ")),
        Box::new(&b"World"[..]),
        Box::new(Cursor::new(b"!")),
    ];
    let mut readers = MultiReader::new(readers);
    let mut buf = String::new();
    readers.read_to_string(&mut buf)?;
    assert_eq!(buf.as_str(), "Hello, World!");
    Ok(())
}
