// https://users.rust-lang.org/t/removing-multiple-indices-from-a-vector/65599/3
// clean way to remove n sorted indicies from a vec without O(n^2)
pub fn remove_sorted_indices<T>(
    v: impl IntoIterator<Item = T>,
    indices: impl IntoIterator<Item = usize>,
) -> Vec<T> {
    let v = v.into_iter();
    let mut indices = indices.into_iter();
    let mut i = match indices.next() {
        None => return v.collect(),
        Some(i) => i,
    };
    let (min, max) = v.size_hint();
    let mut result = Vec::with_capacity(max.unwrap_or(min));

    for (j, x) in v.into_iter().enumerate() {
        if j == i {
            if let Some(idx) = indices.next() {
                i = idx;
            }
        } else {
            result.push(x);
        }
    }

    result
}

// if swap remove satisfies precondition
pub fn swap_remove_all<T>(vec: &mut Vec<T>, indices: &Vec<usize>) {
    for idx in indices.iter().rev() {
        vec.swap_remove(*idx);
    }
}
