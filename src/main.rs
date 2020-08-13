use std::iter;
use serde::{Deserialize};

#[derive(Debug)]
struct ColumnConfigs<'a> {
    spec: &'a Vec<usize>,
    spaces: Vec<usize>,
    size: usize,
    n_space: usize, 
}

impl <'a> ColumnConfigs<'a> {
    fn remaining_spaces(&self) -> Option<usize> {
        self.n_space.checked_sub(self.spaces.iter().sum::<usize>())
    }

    fn increment(&mut self, remaining_spaces: usize) {
        // reset first non-zero and increment entry after that
        let j = if remaining_spaces==0 {
            let i_max = self.spec.len().checked_sub(1).unwrap_or(0);
            let i = self.spaces.iter().enumerate()
                .filter(|(_i, &s)| s>0).map(|(i, _s)| i).next().unwrap_or(i_max);
            if i<i_max {
                self.spaces[i] = 0;
                i+1
            } else { i } //keep incrementing last entry if we are done
        } else { 0 };  //increment first entry if nothing is reset
        self.spaces[j]+=1;        
    }

    fn make_config_iter(&'a self, remaining_spaces: usize) -> impl Iterator<Item=bool> + 'a {
        self.spaces.iter().enumerate().zip(self.spec.iter())
        .flat_map(|((i, &s),&t)| {
            iter::repeat(false).take(if i==0 {s} else {s+1}).chain(iter::repeat(true).take(t))
        })
        .chain(iter::repeat(false).take(remaining_spaces))
    }

    fn make_config(&self, remaining_spaces: usize) -> Vec<bool> {
        self.make_config_iter(remaining_spaces).collect()
    }
}

impl Iterator for ColumnConfigs<'_> {
    type Item = Vec<bool>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(remaining_spaces) = self.remaining_spaces() {
            let this_config = self.make_config(remaining_spaces);
            self.increment(remaining_spaces);
            Some(this_config)
        } else {
            None
        }
    }
}

fn column_configs<'a>(spec: &'a Vec<usize>, size: usize) -> ColumnConfigs<'a> {
    assert!(spec.len()>0);
    ColumnConfigs{
        spec, size, 
        spaces: vec![0; spec.len()], 
        n_space: (size+1).checked_sub(spec.len()+spec.iter().sum::<usize>()).expect("Provided size below min requirement")
    }
}

#[test]
fn test_column_configs() {
    assert_eq!(column_configs(&vec![2,2], 6).count(), 3)
}

#[derive(Debug, Deserialize)]
struct Puzzle {
    horizontal: Vec<Vec<usize>>,
    vertical: Vec<Vec<usize>>,
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len()>1 {
        let puzzle: Puzzle = serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
        println!("{:?}", puzzle);
    }
    for cfg in column_configs(&vec![2,2], 6) {
        println!("Possible config: {:?}", cfg);
    }
}
