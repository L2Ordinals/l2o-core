#[derive(Copy, Clone)]
pub enum Statistic {
    Schema = 0,
    BlessedInscriptions = 1,
    Commits = 2,
    CursedInscriptions = 3,
    IndexRunes = 4,
    IndexSats = 5,
    LostSats = 6,
    OutputsTraversed = 7,
    ReservedRunes = 8,
    Runes = 9,
    SatRanges = 10,
    UnboundInscriptions = 11,
    IndexTransactions = 12,
}

impl Statistic {
    pub fn key(self) -> u64 {
        self.into()
    }
}

impl From<Statistic> for u64 {
    fn from(statistic: Statistic) -> Self {
        statistic as u64
    }
}
