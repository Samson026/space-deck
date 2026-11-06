#[derive(Default)]
pub struct Progress {
    pub progress: u64,
    pub total: u64,
    pub errors: Vec<String>,
}
