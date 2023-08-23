use deku::prelude::*;
use serde::{Deserialize, Serialize, Serializer};
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize, DekuRead)]
#[deku(type = "u16", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[serde(tag = "type")]
enum Type {
    #[deku(id = "0")]
    Human,

    #[deku(id_pat = "_")]
    Cpu {
        #[deku(map = "|v: u16| -> Result<_, DekuError> { Ok(v - 1) }")]
        level: u16,
    },
}

#[derive(Debug, Serialize, Deserialize, DekuRead)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
struct Worm {
    #[deku(until = "|v: &u8| *v == 0", map = "Worm::map_name")]
    name: String,

    #[deku(skip, cond = "name.len() % 2 != 0")]
    #[serde(skip)]
    #[allow(dead_code)]
    padding: Option<u8>,

    played: u16,
    kills_for: u16,
}

impl Worm {
    fn map_name(bytes: Vec<u8>) -> Result<String, DekuError> {
        CString::from_vec_with_nul(bytes)
            .map_err(|_| DekuError::Parse("CString".to_string()))
            .and_then(|v| {
                v.into_string()
                    .map_err(|_| DekuError::Parse("String".to_string()))
            })
    }
}

#[derive(Debug, Serialize, Deserialize, DekuRead)]
#[deku(endian = "big", magic = b"WRM2TEAM")]
struct Team {
    team_type: Type,
    worm_health: u16,
    played: u16,
    won: u16,
    kills_for: u16,
    kills_against: u16,

    #[deku(count = "8")]
    #[serde(serialize_with = "Team::filter_worms")]
    worms: Vec<Worm>,
}

impl Team {
    fn filter_worms<S>(worms: &[Worm], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let filtered_worms: Vec<&Worm> = worms.iter().filter(|w| !w.name.is_empty()).collect();
        filtered_worms.serialize(serializer)
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let (_, team) = Team::from_bytes((&buffer, 0)).unwrap();

    // println!("{:02X?}", buffer);
    // println!("{:#?}", team);

    let json = serde_json::to_string_pretty(&team).unwrap();

    println!("{}", json);

    Ok(())
}
