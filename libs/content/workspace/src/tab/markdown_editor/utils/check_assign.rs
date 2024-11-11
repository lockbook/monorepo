/// Assigns *current = new and returns whether the value changed.
pub fn check_assign<T: PartialEq>(current: &mut T, new: T) -> bool {
    if *current != new {
        *current = new;
        true
    } else {
        false
    }
}
