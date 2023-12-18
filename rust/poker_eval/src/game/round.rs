#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Round {
    Preflop,
    Flop,
    Turn,
    River,
}

