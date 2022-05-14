// if swap remove satisfies precondition
pub fn swap_remove_all<T>(vec: &mut Vec<T>, indices: &Vec<usize>) {
    for idx in indices.iter().rev() {
        vec.swap_remove(*idx);
    }
}
