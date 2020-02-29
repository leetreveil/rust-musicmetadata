extern crate byteorder;
extern crate simple_error;

use std::str;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;


use crate::common::*;
// use crate::common::RecCount;


pub fn parse(file_path: &str) -> Result<Metadata, Box<dyn std::error::Error + 'static>> {
    let mut file = File::open(file_path)?;
    const CHUNK_SIZE: isize = 128;

    file.seek(SeekFrom::End(-CHUNK_SIZE as i64))?;

    let mut data = [0; CHUNK_SIZE as usize];
    file.read_exact(&mut data)?;

    let header = &data[0..3];
    if header != b"TAG" {
        bail!("No TAG header found");
    }

    fn clean (i: &[u8]) -> Result<String, Box<std::error::Error>> {
      return Ok(String::from_utf8(i.to_vec())?.trim_end_matches(char::from(0)).to_owned());
    }

    let mut metadata = Metadata::default();

    metadata.title = clean(&data[3..33])?;
    metadata.artist = vec![clean(&data[33..63])?];
    metadata.album = clean(&data[63..93])?;
    metadata.year = clean(&data[93..97])?;
    metadata.comment = vec![clean(&data[97..125])?];

    //TODO: both track and disk should be optional?
    metadata.track = RecCount{ no: if data[126] == 0 { 0 } else { data[126] }, of: 0};
    // let disk = RecCount{ no: 0, of: 0 };

    // TODO: map over to the number if the lookup fails?
    metadata.genre = GENRES.get(data[127] as usize)
                      .map(|s| s.to_string())
                      .into_iter().collect();

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id3v1() {
      let m = parse("assets/id3v1.mp3").unwrap();
      assert_eq!(m.title, "Blood Sugar");
      assert_eq!(m.artist, vec!["Pendulum"]);
      assert_eq!(m.album, "Blood Sugar (Single)");
      assert_eq!(m.year, "2007");
      assert_eq!(m.comment, vec!["abcdefg"]);
      assert_eq!(m.track, RecCount{no: 1, of: 0});
      assert_eq!(m.disk, None);
      assert_eq!(m.genre, vec!["Electronic"]);
      assert_eq!(m.picture, vec!());
    }

}