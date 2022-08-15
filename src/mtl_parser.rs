use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone)]
pub struct MtlData {
    pub texture_path_map: HashMap<String, String>,
}

impl MtlData {
    pub fn new(mtl_path: &str) -> MtlData {
        let file = File::open(mtl_path).unwrap();
        let reader = BufReader::new(file);
        let mut texture_path_map: HashMap<String, String> = HashMap::new();

        let mut current_mtl_name: Option<String> = None;

        for line in reader.lines() {
            let line = line.unwrap();
            if line == "" {
                continue;
            }
            let elements = line.split_whitespace().collect::<Vec<&str>>();
            let id = elements[0];
            match id {
                "newmtl" => current_mtl_name = Some(elements[1].to_string()),
                "map_Kd" => {
                    //FIXME: is .clone.unwrap an anti pattenrn?
                    texture_path_map
                        .insert(current_mtl_name.clone().unwrap(), elements[1].to_string());
                }
                _ => println!("Unhandled .mtl expression: {}", line),
            }
        }
        MtlData { texture_path_map }
    }
}
