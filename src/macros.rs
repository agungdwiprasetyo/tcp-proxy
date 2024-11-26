#[macro_export]
macro_rules! try_or_skip {
    ($r:expr) => {
        if let Ok(val) = $r { val } 
        else { return }
    };
}
