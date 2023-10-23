#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(pub usize);

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
