use std::collections::HashSet;
use std::{env, fs};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use spreadsheet_ods::OdsOptions;

fn main() {
    let file = env::args()
        .nth(1)
        .expect("Please provide an ods file to convert");

    let path_to_file = PathBuf::from(file);
    let input = BufReader::new(File::open(&path_to_file).expect("Could not open ods file"));

    let wb = OdsOptions::default().
        // don't read styles
        content_only()
        // don't create empty cells
        .ignore_empty_cells()
        .read_ods(input).expect("Could not parse ods file");

    let max_rows = 1000u32;

    let mut shape_set: HashSet<String> = HashSet::new();

    // write states csv
    let states_sheet_index = wb.sheet_idx("states").expect("A sheet with name 'states' is required.");
    let states_sheet = wb.sheet(states_sheet_index);

    let states_csv_path = path_to_file.with_extension("states.csv");
    let mut states_output = BufWriter::new(File::create(&states_csv_path).expect("Could not create/truncate states file"));
    states_output.write_all(b"\"name\",\"description\",\"shape\"\n").unwrap();

    // parse the states, duplicate the ones with multiple shapes
    let row_iter_states = states_sheet.iter_rows((1, 0)..(max_rows, 3));
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

    // write steps csv
    let steps_sheet_index = wb.sheet_idx("steps").expect("A sheet with name 'steps' is required.");
    let steps_sheet = wb.sheet(steps_sheet_index);

    let steps_csv_path = path_to_file.with_extension("steps.csv");
    let mut steps_output = BufWriter::new(File::create(&steps_csv_path).expect("Could not create/truncate steps file"));
    steps_output.write_all(b"\"name\",\"description\",\"level\",\"start_state\",\"end_state\"\n").unwrap();

    let row_iter_steps = steps_sheet.iter_rows((1, 0)..(max_rows, 5));
    for ((_row, col), value) in row_iter_steps {
        if col == 4 {
            steps_output.write_all(b"\"").unwrap();
            steps_output.write_all(value.value.as_str_opt().unwrap().as_bytes()).unwrap();
            steps_output.write_all(b"\"\n").unwrap();
        } else {
            steps_output.write_all(b"\"").unwrap();
            steps_output.write_all(value.value.as_str_opt().unwrap().as_bytes()).unwrap();
            steps_output.write_all(b"\",").unwrap();
        }
    }

    // write shapes csv
    let mut shapes_str = String::from("name\n");
    shapes_str.push_str(shape_set.iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>().join("\n").as_str());
    shapes_str.push('\n');
    let shapes_csv_path = path_to_file.with_extension("shapes.csv");
    let shapes_ttl_path = path_to_file.with_extension("shapes.ttl");
    let mut shapes_output_file = File::create(&shapes_csv_path).expect("Could not create/truncate shapes file");
    shapes_output_file.write_all(shapes_str.as_bytes()).unwrap();

    // write the RML mapping file
    let states_ttl_path = path_to_file.with_extension("states.ttl");
    
    let rml_mappings = fs::read_to_string(Path::new("resources").join("mapping-template.rml.ttl")).expect("Could not read mapping-template.rml.ttl file")
        .replacen("@@SHAPES.CSV@@", &shapes_csv_path.to_str().unwrap(), 1)
        .replacen("@@SHAPES.TTL@@", &shapes_ttl_path.to_str().unwrap(), 1)
        .replacen("@@STATES.CSV@@", &states_csv_path.to_str().unwrap(), 1)
        .replacen("@@STATES.TTL@@", &states_ttl_path.to_str().unwrap(), 1);
    let rml_mapping_output_path = path_to_file.with_extension("mapping.rml.ttl");
    fs::write(rml_mapping_output_path, rml_mappings).expect("Could not write rml mappings file");
}
