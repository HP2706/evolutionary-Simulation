

pub struct PointMutation {
    // attributes: mutation_rate
    mutation_rate: f64, // 2 x 10^-5 per symbol
}

const MUTATION_RATE: f64 = 1e-5; // 10^-5

pub struct GeneDuplication {
    mutation_rate: f64,
}

pub struct SplitMutation {
    mutation_rate: f64,
}

impl SplitMutation {
    pub fn new(mutation_rate: f64) -> SplitMutation {
        SplitMutation {
            mutation_rate: mutation_rate,
        }
    }
}

impl PointMutation {
    pub fn new(mutation_rate: f64) -> PointMutation {
        PointMutation {
            mutation_rate: mutation_rate,
        }
    }
}

impl GeneDuplication {
    pub fn new(mutation_rate: f64) -> GeneDuplication {
        GeneDuplication {
            mutation_rate: mutation_rate,
        }
    }
}


pub enum Mutation {
    PointMutation(PointMutation),
    GeneDuplication(GeneDuplication),
    SplitMutation(SplitMutation),
}