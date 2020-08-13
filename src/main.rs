use std::iter;

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

fn column_configs_as_option<'a>(spec: &'a Vec<usize>, size: usize) -> Option<ColumnConfigs<'a>> {
    let spaces: Vec<_> = iter::repeat(0).take(spec.len()).collect();
    let maybe_n_space = (size+1).checked_sub(spec.len()+spec.iter().sum::<usize>());
    if let Some(n_space) = maybe_n_space {
        Some(ColumnConfigs{spec, spaces, n_space})
    } else { None }
}

/// Return iterator over all possible configurations of blocks of length
/// as specified in spec. Each configuration is given as a `Vec<bool>` of length
/// size.
/// If requested size cannot be satisfied, an empty iterator is returned.
fn column_configs_as_empty<'a>(spec: &'a Vec<usize>, size: usize) -> 
impl Iterator<Item = Vec<bool>> + 'a {
    column_configs_as_option(spec, size)
    .into_iter().flat_map(move |some_iter| some_iter)
}

fn main() {
    for cfg in column_configs_as_empty(&vec![2,2], 6) {
        println!("Possible config: {:?}", cfg);
    }
}
