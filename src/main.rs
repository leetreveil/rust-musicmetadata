extern crate byteorder;
extern crate simple_error;

mod common;
mod id3v1;
mod id4;


fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let _out = id4::parse("assets/id4.m4a")?;
    let out = id3v1::parse("assets/id3v1.mp3")?;
    println!("{:?}", out);
    Ok(())
}

