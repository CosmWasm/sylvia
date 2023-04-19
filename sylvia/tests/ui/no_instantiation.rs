use sylvia::contract;

pub struct Sth {}

#[contract]
impl Sth {
    pub fn new() -> Self {
        Sth {}
    }
}

fn main() {}
