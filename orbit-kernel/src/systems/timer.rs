pub trait Timer {
    fn set(&self, ticks: usize);
    fn get(&self) -> usize;
}
