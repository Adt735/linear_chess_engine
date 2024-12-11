use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};
use std::fs::File;

use crate::eval::MATERIAL_SCORE;


#[derive(Serialize, Deserialize)]
pub struct EngineValues {
    pub material_score:[i32;6],
}


fn read_from_json_file(filename: &str) -> EngineValues {
    let file = File::open(filename).expect("File not found");
    let reader = std::io::BufReader::new(file);
    let person: EngineValues = from_reader(reader).expect("JSON mal formatted");
    person
}

pub fn save_to_json_file<T>(data: &T, filename: &str) -> std::io::Result<()>
where
    T: serde::Serialize,
{
    let file = File::create(filename)?;
    to_writer(file, data)?;
    Ok(())
}


pub unsafe fn init_eval_constants(filename: &str) {
    let values = read_from_json_file(filename);

    MATERIAL_SCORE[..6].copy_from_slice(&values.material_score);
    for i in 6..12 {
        MATERIAL_SCORE[i] = -values.material_score[i-6];
    }
}