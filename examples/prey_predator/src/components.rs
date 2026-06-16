#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Species {
    Prey,
    Predator,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub species: Species,
}
