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
}

struct ColumnConfigs<'a> {
    spec: &'a BlockSpec,
    /// spaces is same length as block_sizes and indicate number spaces before a given block, not including spaces
    /// that must be present. 
    /// remaining_spaces is redundant information only passed for efficiency
    spaces: Vec<usize>,
    exhausted: bool, // needed to allow 0-bloc configs
}

// the spaces vectors could also be generated as iterators. that will be next step...
impl <'a> ColumnConfigs<'a> {
    fn new(spec: &'a BlockSpec) -> Self {
        ColumnConfigs{spec, spaces: vec![0; spec.block_sizes.len()], exhausted: false}
    }

    fn remaining_spaces(&self) -> usize {
        self.spec.n_space.checked_sub(self.spaces.iter().sum::<usize>()).expect("Invalid spaces state")
    }

    fn increment(&mut self) {
        assert!(!self.exhausted);
        let n_remain = self.remaining_spaces();
        if n_remain > 0 {
            // Add more more space at start
            self.spaces[0]+=1;
            return
        } 
        if let Some(i) = self.spaces.iter().enumerate()
            .filter(|(_i, &s)| s>0).map(|(i, _s)| i).next() {
                if i+1<self.spaces.len() {
                    // reset first non-zero and increment the next entry
                    self.spaces[i+1]+=1;
                    self.spaces[i]=0;
                    return 
                }
        } 
        // we are done
        self.exhausted = true;
    }

    fn make_config_iter(&'a self) -> impl Iterator<Item=bool> + 'a {
        assert!(!self.exhausted);
        let n_remain = self.remaining_spaces();
        self.spaces.iter().enumerate().zip(self.spec.block_sizes.iter())
        .flat_map(|((i, &s),&t)| {
            iter::repeat(false).take(if i==0 {s} else {s+1}).chain(iter::repeat(true).take(t))
        })
        .chain(iter::repeat(false).take(n_remain))
    }
}

#[test]
fn test_block_spec() {
    let bs = BlockSpec::new(vec![2,3], 7);
    assert_eq!(bs.n_space, 7-(2+1+3));
}

fn all_configs(bs: &BlockSpec) -> Vec<Vec<bool>> {
    let mut cc = ColumnConfigs::new(&bs);
    let mut configs = Vec::new();
    while !cc.exhausted  {
        configs.push(cc.make_config_iter().collect());
        cc.increment();
    };
    configs
}

#[test]
fn test_column_configs() {
    let bs = BlockSpec::new(vec![2,3], 7);
    let mut cc = ColumnConfigs::new(&bs);
    let mut n = 0;
    while !cc.exhausted  {
        n+=1;
        cc.increment();
    };
    assert_eq!(n, 3);
}

#[test]
fn test_wider_configs() {
    assert_eq!(all_configs(&BlockSpec::new(vec![2], 7)).len(), 6);
    assert_eq!(all_configs(&BlockSpec::new(vec![2,2], 7)).len(), 6);
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
    if cols.len()==0 {return Some(Vec::new())};
    let col = &cols[0];
    let rest = &cols[1..];
    let mut cc = ColumnConfigs::new(col);
    while !cc.exhausted {
        if let Ok(next_row_configs) = cc.make_config_iter().zip(row_configs.iter())
        .map(|(c, row)| advance_row(row, c, rest.len()))
        .collect() {
            if let Some(mut sol)=solve_recursive(next_row_configs, rest) {
                sol.push(ColumnConfigs::new(col).make_config_iter().collect());
                return Some(sol)
            }
        };
        cc.increment();
    };
    None
}

fn make_row_config(bs: &BlockSpec) -> Vec<bool> {
    let cc = ColumnConfigs::new(bs);
    iter::once(false)
    .chain(cc.make_config_iter().take(bs.size-cc.remaining_spaces()))
    .chain(iter::once(false))
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
    println!("Solution: {:?}", res);
}
