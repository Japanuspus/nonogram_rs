use std::iter;

#[derive(Debug)]
struct ColumnConfigs<'a> {
    spec: &'a Vec<usize>,
    spaces: Vec<usize>,
    n_space: usize,
}

impl Iterator for ColumnConfigs<'_> {
    type Item = Vec<bool>;
    fn next(&mut self) -> Option<Self::Item> {
        None // logic goes here
    }
}

fn column_configs<'a>(spec: &'a Vec<usize>, size: usize) -> ColumnConfigs<'a> {
    ColumnConfigs{
        spec, 
        spaces: vec![0; spec.len()], 
        n_space: (size+1).checked_sub(spec.len()+spec.iter().sum::<usize>()).expect("Provided size below min requirement")
    }
}

fn main() {
    for cfg in column_configs(&vec![2,2], 6) {
        println!("Possible config: {:?}", cfg);
    }
}
