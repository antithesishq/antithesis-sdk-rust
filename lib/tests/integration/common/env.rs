use std::env;

pub fn set_var(k: &str, v: &str) -> Option<String> {
    let prev_v = env::var(k);
    env::set_var(k, v);
    match prev_v {
        Ok(s) => Some(s),
        Err(_) => None
    }
}

pub fn restore_var(k: &str, maybe_v: Option<String>) {
    match maybe_v {
        None => env::remove_var(k),
        Some(prev_v) => env::set_var(k, prev_v.as_str())
    };
}
