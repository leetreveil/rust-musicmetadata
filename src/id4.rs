extern crate byteorder;
extern crate simple_error;

use std::str;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::borrow::Cow;

use byteorder::{ReadBytesExt, BigEndian};

use crate::common::*;
use crate::common::RecCount;


static CONTAINER_ATOMS: &[&[u8; 4]] = &[b"moov", b"udta", b"meta", b"ilst", b"trak", b"mdia"];


pub fn parse(file_path: &str) -> Result<Metadata, Box<dyn std::error::Error + 'static>> {
  let mut file = File::open(file_path)?;

  // skip the first atom (ftyp)
  let atom_length = file.read_u32::<BigEndian>()?;

  let mut atom_name = [0; 4];
  file.read_exact(&mut atom_name)?;

  file.seek(SeekFrom::Current(i64::from(atom_length - 8)))?;

  let mut parent_atom = Cow::Borrowed(b"unko");
  let mut parent_atom_length = 0;
  let mut total_meta_atom_length = 0;

  let mut abc = Metadata::default();
  
  loop {
    let atom_length = file.read_u32::<BigEndian>()?;

    let mut atom_name = [0; 4];
    file.read_exact(&mut atom_name)?;
    println!("atom_length={:?}, atom_name={:?}, parent_atom={:?}, parent_atom_length={:?}", atom_length, String::from_utf8_lossy(&atom_name), String::from_utf8_lossy(parent_atom.to_mut()), parent_atom_length);
    // println!("atom_length={:?}, atom_name={:?}, parent_atom={:?}, parent_atom_length={:?}", atom_length, String::from_utf8_lossy(&atom_name), atom_name, String::from_utf8_lossy(parent_atom.to_mut()), parent_atom_length);

    if parent_atom.to_mut() == b"ilst" {
      total_meta_atom_length += atom_length;

      // skip proprietary apple atoms (our parser cannot handle them) 
      if &atom_name == b"----"{
        println!("Hit the stupid apple atom");
        file.seek(SeekFrom::Current(i64::from(atom_length - 8)))?;
        continue;
      }

      let mut sub_len = 0;

      while sub_len < atom_length {
        let len = file.read_u32::<BigEndian>()?; 
        println!(">> len={:?}", len);

        sub_len += len + 8;

        // not sure what this is but it always reads as 'data'
        let mut wut = [0; 4];
        file.read_exact(&mut wut)?;

        let dtype = file.read_u32::<BigEndian>()?; 
        let dtype_str = match dtype {
          0 => "uint8",
          1 => "text",
          13 => "jpeg",
          14 => "png",
          21 => "uint8",
          _ => "unknown"
        };

        println!(">> dtype_str={:?}", dtype_str);
        
        // skip 4 bytes padding
        file.seek(SeekFrom::Current((4) as i64))?;

        // read the data
        let mut whatwewant = vec![0u8; (len - 16) as usize];
        file.read_exact(&mut whatwewant)?;

        match &atom_name {
          b"covr" => abc.picture.push(Picture{ format: "image/".to_owned() + dtype_str, data: whatwewant }),
          b"trkn" => abc.track = RecCount{ no: whatwewant[3], of: whatwewant[5] },
          b"disk" => abc.disk = Some(RecCount{ no: whatwewant[3], of: whatwewant[5] }),
          b"\xA9alb" => abc.album = String::from_utf8(whatwewant)?,
          b"\xA9ART" => abc.artist = vec![String::from_utf8(whatwewant)?],
          b"\xA9cmt" => abc.comment = vec![String::from_utf8(whatwewant)?],
          b"\xA9day" => abc.year = String::from_utf8(whatwewant)?,
          b"\xA9nam" => abc.title = String::from_utf8(whatwewant)?,
          b"gnre" => abc.genre = GENRES.get((whatwewant[1] - 1) as usize).map(|s| s.to_string()).into_iter().collect(),
          _ => ()
        }
      }

      if total_meta_atom_length >= parent_atom_length - 8 {
        break
      }
      continue
    }

    if &atom_name == b"meta" {
      file.seek(SeekFrom::Current(4))?;
      continue
    }

    if CONTAINER_ATOMS.contains(&&atom_name) {
      parent_atom = Cow::Owned(atom_name);
      parent_atom_length = atom_length;
      continue
    } else {
      file.seek(SeekFrom::Current(i64::from(atom_length - 8)))?;
    }
  }

  Ok(abc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id4() {
      let m = parse("assets/id4.m4a").unwrap();
      assert_eq!(m.title, "Voodoo People (Pendulum Remix)");
      assert_eq!(m.artist, vec!["The Prodigy"]);
      assert_eq!(m.album, "Voodoo People");
      assert_eq!(m.year, "2005");
      assert_eq!(m.track, RecCount{no: 1, of: 12});
      assert_eq!(m.disk, Some(RecCount{no: 1, of: 1}));
      assert_eq!(m.genre, vec!["Electronic"]);
      assert_eq!(m.picture.len(), 2);
      assert_eq!(m.picture[0].format, "image/jpeg");
      assert_eq!(m.picture[0].data.len(), 196450);
      assert_eq!(m.picture[1].format, "image/jpeg");
      assert_eq!(m.picture[1].data.len(), 196450);
    }
}
