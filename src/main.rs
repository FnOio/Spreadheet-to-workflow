use std::collections::HashSet;
use std::{env, fs};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use spreadsheet_ods::{OdsOptions, WorkBook};
use serde::{Deserialize, Serialize};

const MAX_ROWS:u32 = 1000;

fn main() {
    let file = env::args()
        .nth(1)
        .expect("Please provide an ods file to convert");

    let path_to_file = PathBuf::from(file).canonicalize().unwrap();
    let input = BufReader::new(File::open(&path_to_file).expect("Could not open ods file"));

    let wb = OdsOptions::default()
        .content_only()// only data, no styles, formulas...
        .read_ods(input).expect("Could not parse ods file");

    // output files go to a directory with name of the spreadsheet file (without extension)
    let output_path_suffix = format!("{}_output", path_to_file.file_stem().unwrap().to_str().unwrap());
    let output_path = path_to_file.parent().unwrap().join(output_path_suffix);
    fs::create_dir_all(&output_path).unwrap();

    write_states(&wb, &output_path);
    write_steps(&wb, &output_path);
    write_mappings(&output_path);
}

fn write_states(workbook: &WorkBook, output_path: &PathBuf) {
    // shapes found during extracting the states will be saved in this set and returned
    let mut shape_set: HashSet<String> = HashSet::new();

    // find the states sheet in the workbook
    let states_sheet_index = workbook.sheet_idx("states").expect("A sheet with name 'states' is required.");
    let states_sheet = workbook.sheet(states_sheet_index);

    // this is where the states will be written as CSV
    let states_csv_path = output_path.join("states.csv");
    let mut states_output = BufWriter::new(File::create(&states_csv_path).expect("Could not create/truncate states file"));
    states_output.write_all(b"\"name\",\"description\",\"shape\"\n").unwrap();

    // parse the states, duplicate the ones with multiple shapes
    let row_iter_states = states_sheet.iter_rows((1, 0)..(MAX_ROWS, 3));
    let mut current_output_line = String::new();
    for ((_row, col), value) in row_iter_states {
        if col == 2 {   // This is the column with shapes. If multiple shapes are listes, the lines are duplicated with a single shape per line.
            value.value.as_str_opt().unwrap()
                .replace(' ', "")
                .split(',')
                .for_each(|shape| {
                    states_output.write_all(current_output_line.as_bytes()).unwrap();
                    states_output.write_all(b"\"").unwrap();
                    states_output.write_all(shape.as_bytes()).unwrap();
                    states_output.write_all(b"\"\n").unwrap();
                    shape_set.insert(shape.to_string());
                });
            current_output_line.clear();
        } else {
            current_output_line.push_str("\"");
            current_output_line.push_str(value.value.as_str_opt().unwrap());
            current_output_line.push_str("\",");
        }
    }

    // now write the shapes
    let mut shapes_str = String::from("name\n");
    shapes_str.push_str(shape_set.iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>().join("\n").as_str());
    shapes_str.push('\n');
    let shapes_csv_path = output_path.join("shapes.csv");
    let mut shapes_output_file = File::create(&shapes_csv_path).expect("Could not create/truncate shapes file");
    shapes_output_file.write_all(shapes_str.as_bytes()).unwrap();
}

#[derive(Serialize, Deserialize)]
struct Step {
    name: String,
    description: String,
    levels: HashSet<String>,
    start_states: HashSet<String>,
    end_states: HashSet<String>,
}

fn write_steps(workbook: &WorkBook, output_path: &PathBuf) {
    // find the steps sheet in the workbook
    let steps_sheet_index = workbook.sheet_idx("steps").expect("A sheet with name 'steps' is required.");
    let steps_sheet = workbook.sheet(steps_sheet_index);

    // this is where the steps will be written as CSV
    let steps_json_path = output_path.join("steps.json");
    let mut steps_output = BufWriter::new(File::create(&steps_json_path).expect("Could not create/truncate steps file"));
    //steps_output.write_all(b"\"name\",\"description\",\"level\",\"start_state\",\"end_state\"\n").unwrap();

    let row_iter_steps = steps_sheet.iter_rows((1, 0)..(MAX_ROWS, 5));

    let mut name: String = String::new();
    let mut description: String = String::new();
    let mut levels: HashSet<String> = HashSet::new();
    let mut start_states: HashSet<String> = HashSet::new();
    
    let mut json_steps: Vec<Step> = Vec::new();

    for ((_row, col), value) in row_iter_steps {
        match col {
            0 => {
                let value_opt = value.value.as_string_opt();
                match value_opt {
                    Some(value) => {name = value}
                    None => break   // this means an empty row, so stop parsing rows
                }
            },
            1 => description = value.value.as_string_opt().unwrap().to_string(),
            2 => levels =
                value.value.as_str_or_default()
                    .replace(' ', "")
                    .split(',')
                    .map(String::from)
                    .collect(),
            3 => {
                let start_states_option = value.value.as_string_opt();
                start_states = match start_states_option { 
                    Some(start_states) => {
                        start_states
                            .replace(' ', "")
                            .split(',')
                            .map(String::from)
                            .collect()
                    },
                    None => HashSet::new(),
                };
            },
            4 => {
                let end_states: HashSet<String> = value.value.as_string_opt().unwrap().to_string()
                    .replace(' ', "")
                    .split(',')
                    .map(String::from)
                    .collect();
                let step = Step {
                    name: name.clone(), 
                    description: description.clone(), 
                    levels: levels.clone(), 
                    start_states: start_states.clone(),
                    end_states
                };
                json_steps.push(step);
            },
            _ => { /* ignore the rest of the columns */}
        }
    }
    serde_json::to_writer(&mut steps_output, &json_steps).unwrap();
}

fn write_mappings(output_path: &PathBuf) {
    // write the YARRRML mapping file
    let states_ttl_path = output_path.join("states.ttl");
    let states_csv_path = output_path.join("states.csv");
    let steps_ttl_path = output_path.join("steps.ttl");
    let steps_json_path = output_path.join("steps.json");
    let shapes_csv_path = output_path.join("shapes.csv");
    let shapes_ttl_path = output_path.join("shapes.ttl");
    let mapping_output_path = output_path.join("mapping.yarrrml.yaml");

    let yarrrml_mappings = fs::read_to_string(Path::new("resources").join("mapping-template.yarrrml.yaml")).expect("Could not read mapping-template.yarrrml.yaml file")
        .replacen("@@SHAPES.CSV@@", &shapes_csv_path.to_str().unwrap(), 1)
        .replacen("@@SHAPES.TTL@@", &shapes_ttl_path.to_str().unwrap(), 1)
        .replacen("@@STATES.CSV@@", &states_csv_path.to_str().unwrap(), 1)
        .replacen("@@STATES.TTL@@", &states_ttl_path.to_str().unwrap(), 1)
        .replacen("@@STEPS.JSON@@", &steps_json_path.to_str().unwrap(), 1)
        .replacen("@@STEPS.TTL@@", &steps_ttl_path.to_str().unwrap(), 1);
    fs::write(mapping_output_path, yarrrml_mappings).expect("Could not write mappings file");
}
