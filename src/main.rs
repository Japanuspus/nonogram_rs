use std::iter;
use serde::{Deserialize};


struct BlockSpec {
    block_sizes: Vec<usize>,
    size: usize,
    n_space: usize,
}

impl BlockSpec {
    fn new(block_sizes: Vec<usize>, size: usize) -> Self {
        let n_space = (size+1).checked_sub(block_sizes.len()+block_sizes.iter().sum::<usize>()).expect("Provided size below min requirement");
        Self{block_sizes, size, n_space}
    }

    /// spaces is same length as block_sizes and indicate number spaces before a given block, not including spaces
    /// that must be present. 
    /// remaining_spaces is redundant information only passed for efficiency
    fn make_config_iter<'a>(&'a self, spaces: &'a Vec<usize>, remaining_spaces: usize) -> impl Iterator<Item=bool> + 'a {
        spaces.iter().enumerate().zip(self.block_sizes.iter())
        .flat_map(|((i, &s),&t)| {
            iter::repeat(false).take(if i==0 {s} else {s+1}).chain(iter::repeat(true).take(t))
        })
        .chain(iter::repeat(false).take(remaining_spaces))
    }

    fn make_config(&self, spaces: &Vec<usize>, remaining_spaces: usize) -> Vec<bool> {
        self.make_config_iter(spaces, remaining_spaces).collect()
    }
}

struct ColumnConfigs<'a> {
    spec: &'a BlockSpec,
    spaces: Vec<usize>,
}

impl <'a> ColumnConfigs<'a> {
    fn remaining_spaces(&self) -> Option<usize> {
        self.spec.n_space.checked_sub(self.spaces.iter().sum::<usize>())
    }

    fn increment(&mut self, remaining_spaces: usize) {
        // reset first non-zero and increment entry after that
        let j = if remaining_spaces==0 {
            let i_max = self.spec.block_sizes.len().checked_sub(1).unwrap_or(0);
            let i = self.spaces.iter().enumerate()
                .filter(|(_i, &s)| s>0).map(|(i, _s)| i).next().unwrap_or(i_max);
            if i<i_max {
                self.spaces[i] = 0;
                i+1
            } else { i } //keep incrementing last entry if we are done
        } else { 0 };  //increment first entry if nothing is reset
        self.spaces[j]+=1;        
    }
}

impl Iterator for ColumnConfigs<'_> {
    type Item = Vec<bool>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(remaining_spaces) = self.remaining_spaces() {
            let this_config = self.spec.make_config(&self.spaces, remaining_spaces);
            self.increment(remaining_spaces);
            Some(this_config)
        } else {
            None
        }
    }
}

fn column_configs<'a>(spec: &'a BlockSpec) -> ColumnConfigs<'a> {
    ColumnConfigs{spec, spaces: vec![0; spec.block_sizes.len()]}
}

#[test]
fn test_block_spec() {
    let bs = BlockSpec::new(vec![2,3], 7);
    assert_eq!(bs.n_space, 7-(2+1+3));
}

#[test]
fn test_column_configs() {
    let bs = BlockSpec::new(vec![2,3], 7);
    assert_eq!(column_configs(&bs).count(), 3)
}

/// Input file structure
#[derive(Debug, Deserialize)]
struct Puzzle {
    horizontal: Vec<Vec<usize>>,
    vertical: Vec<Vec<usize>>,
}

/// first entry of row is most recent output. return a slice starting with output to satisfy col
/// remaining column count is number of columns to output after this
fn advance_row(row: &[bool], col: bool, remaining_column_count: usize) -> Result<&[bool], ()> {
    let r0 = row[0];
    if r0 {
        // last output was a mark and we are forced to advance. 
        // row[1] is always defined in this case
        if row[1]==col {
            Ok(&row[1..])
        } else {
            Err(())
        }
    } else {
        // last output was space
        if col {
            // must output mark.
            // next entry will always be mark -- if it exists
            if row.len()>1 {
                Ok(&row[1..])
            } else {
                Err(())
            }
        } else {
            // not advancing. check remaining column count
            if row.len()<=remaining_column_count+2 {
                Ok(row)
            } else {
                Err(())
            }
        }
    }
}

#[test]
fn test_advance_row() {
    let rr = vec![false, true, true, false, true, false];
    assert_eq!(advance_row(&rr[..], true, 10), Ok(&rr[1..]));
    assert_eq!(advance_row(&rr[..], false, 4), Ok(&rr[..]));
    assert_eq!(advance_row(&rr[..], false, 3), Err(())); // fail on column count

    assert_eq!(advance_row(&rr[1..], true, 10), Ok(&rr[2..]));
    assert_eq!(advance_row(&rr[1..], false, 10), Err(()));
    assert_eq!(advance_row(&rr[2..], true, 10), Err(()));
    assert_eq!(advance_row(&rr[2..], false, 10), Ok(&rr[3..]));
}


fn solve_recursive(row_configs: Vec<&[bool]>, cols: &[BlockSpec]) -> Option<Vec<Vec<bool>>> {
    println!("Recursive solve called for length {}", cols.len());
    if cols.len()==0 {
        return Some(Vec::new())
    }
    let col = &cols[0];
    let rest = &cols[1..];
    for cfg in column_configs(col) {
        if let Ok(next_row_configs) = cfg.iter().zip(row_configs.iter())
        .map(|(&c, row)| advance_row(row, c, rest.len()))
        .collect() {
            if let Some(mut sol)=solve_recursive(next_row_configs, rest) {
                sol.push(cfg);
                return Some(sol)
            }
        }
    };
    None
}

fn make_row_config(bs: &BlockSpec) -> Vec<bool> {
    iter::once(false)
    .chain(bs.make_config_iter(&vec![0;bs.block_sizes.len()], 1))
    .collect()
}

#[test]
fn test_make_row_config() {
    let rc = make_row_config(&BlockSpec::new(vec![2,1], 10));
    assert_eq!(rc, vec![false, true, true, false, true, false]);
}

fn solve(puzzle: Puzzle) -> Option<Vec<Vec<bool>>> {
    let n_col = puzzle.vertical.len();
    let n_row = puzzle.horizontal.len();
    let cols: Vec<_> = puzzle.vertical.into_iter().map(|bs| BlockSpec::new(bs, n_row)).collect();
    let rows: Vec<_> = puzzle.horizontal.into_iter().map(|bs| BlockSpec::new(bs, n_col)).collect();
    let row_configs: Vec<Vec<bool>> = rows.iter().map(make_row_config).collect();
    solve_recursive(row_configs.iter().map(|v| &v[..]).collect(), &cols)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let puzzle: Puzzle = serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    println!("Puzzle: {:?}", puzzle);
    let res = solve(puzzle);
    println!("Solution: {:?}", res)
}
