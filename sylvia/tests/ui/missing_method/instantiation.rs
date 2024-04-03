pub struct Contract {}

#[sylvia::contract]
impl Contract {
    pub const fn new() -> Self {
        Self {}
    }
}

fn main() {}
