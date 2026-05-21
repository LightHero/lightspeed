use rand::RngExt;

/// Returns a random name.
pub fn unique_name() -> String {
    let suffix: u64 = rand::rng().random();
    format!("{suffix:016x}")
}
