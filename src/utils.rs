macro_rules! error {
    // This macro easily performs an early return of an error string
    ($base:expr, $($args:expr),*) => {{
        return Err(format!($base, $($args),*).into());
    }}
}

pub(crate) use error;
