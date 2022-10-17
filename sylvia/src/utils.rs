#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State {
    // Ongoing arrays can be compared to other arrays.
    Ongoing(usize),
    // Finished arrays can only be used by other arrays to be compared to.
    Finished(usize),
    // There can be empty arrays provided and we want to secure ourselves from "index out of bounds"
    Empty,
}

/// # Examples
///
/// Compile time intersection assert.
/// Will panic! in case duplicated messages were provided.
/// Requires sorted arrays to work.
/// ```
///     const _: () = {
///         let msgs: [&[&str]; 2] = [&["msg_a", "msg_b"], &["msg_c", "msg_d"]];
///         sylvia::utils::assert_no_intersection(msgs);
///     };
/// ```
pub const fn assert_no_intersection<const N: usize>(msgs: [&[&str]; N]) {
    let mut states = init_states(&msgs);

    while !should_end(&states) {
        // Get index of array with alphabetically smallest value.
        // This will always be one which state is Ongoing.
        let index = get_next_alphabetical_index(&msgs, &states);

        // Compare all elements at current indexes
        verify_no_collissions(&msgs, &states, &index);

        // Increment index of alaphabeticaly first element
        states[index] = match states[index] {
            State::Ongoing(wi) => {
                if msgs[index].len() == wi + 1 {
                    State::Finished(wi)
                } else {
                    State::Ongoing(wi + 1)
                }
            }
            _ => unreachable!(),
        };
    }
}

const fn init_states<const N: usize>(msgs: &[&[&str]; N]) -> [State; N] {
    let mut states = [State::Ongoing(0); N];
    konst::for_range! {i in 0..N =>
        if msgs[i].is_empty() {
            states[i] = State::Empty;
        }

    }
    states
}

// Finds index of array which is Ongoing and which current value
// is alphabetically smallest
const fn get_next_alphabetical_index<const N: usize>(
    msgs: &[&[&str]; N],
    states: &[State; N],
) -> usize {
    let mut output_index = 0;
    konst::for_range! {i in 0..N =>
        if let State::Ongoing(outer_i) = states[i] {
            match states[output_index] {
                State::Ongoing(inner_i) => {
                    if let std::cmp::Ordering::Greater =
                        konst::cmp_str(msgs[output_index][inner_i], msgs[i][outer_i])
                    {
                        output_index = i;
                    }
                }
                _ => output_index = i,
            }
        }
    }
    output_index
}

// Compare values at current indexes saved in states.
// All comparisions are made with value at index which point to alphabetically smallest
// and values in each other arrays at their current position.
//
// Because arrays are sorted we don't have to compare each value with each other
// and can just compare values in alphabetical ordering.
const fn verify_no_collissions<const N: usize>(
    msgs: &[&[&str]; N],
    states: &[State; N],
    index: &usize,
) {
    let mut i = 0;
    while i < N {
        if i == *index {
            i += 1;
            continue;
        }
        match states[i] {
            State::Ongoing(outer_i) | State::Finished(outer_i) => {
                if let State::Ongoing(inner_i) = states[*index] {
                    if konst::eq_str(msgs[i][outer_i], msgs[*index][inner_i]) {
                        panic!("Message overlaps between interface and contract impl!");
                    }
                }
            }
            _ => (),
        }
        i += 1;
    }
}

const fn should_end<const N: usize>(states: &[State; N]) -> bool {
    konst::for_range! {i in 0..N =>
        if let State::Ongoing(..) = states[i] {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_not_end() {
        let states = [State::Empty, State::Ongoing(3), State::Finished(5)];
        assert!(!super::should_end(&states));
    }

    #[test]
    fn should_end() {
        let states = [State::Empty, State::Finished(3), State::Finished(5)];
        assert!(super::should_end(&states));
    }

    #[test]
    fn init_states() {
        let msgs: [&[&str]; 3] = [&["msg", "msg"], &[], &["msg"]];
        let states = [State::Ongoing(0), State::Empty, State::Ongoing(0)];
        assert_eq!(super::init_states(&msgs), states);
    }

    #[test]
    fn aquire_index_when_two_states_ongoing() {
        let msgs: [&[&str]; 3] = [&["msg_b", "msg_c"], &[], &["msg_a"]];
        let states = [State::Ongoing(1), State::Empty, State::Ongoing(0)];
        assert_eq!(get_next_alphabetical_index(&msgs, &states), 2);
    }

    #[test]
    fn aquire_index_when_mixed_state() {
        let msgs: [&[&str]; 3] = [&["msg_b", "msg_c"], &[], &["msg_a"]];
        let states = [State::Ongoing(1), State::Empty, State::Finished(0)];
        assert_eq!(get_next_alphabetical_index(&msgs, &states), 0);
    }

    #[test]
    fn aquire_index_when_first_array_empty() {
        let msgs: [&[&str]; 3] = [&[], &["msg_b", "msg_c"], &["msg_a"]];
        let states = [State::Empty, State::Ongoing(1), State::Finished(0)];
        assert_eq!(get_next_alphabetical_index(&msgs, &states), 1);
    }

    #[test]
    fn verify_no_collissions() {
        let msgs: [&[&str]; 4] = [&[], &["msg_b", "msg_c"], &["msg_a"], &["msg_d", "msg_a"]];
        let states = [
            State::Empty,
            State::Ongoing(1),
            State::Finished(0),
            State::Ongoing(0),
        ];

        super::verify_no_collissions(&msgs, &states, &1);
        super::verify_no_collissions(&msgs, &states, &3);

        let states = [
            State::Empty,
            State::Ongoing(1),
            State::Finished(0),
            State::Ongoing(1),
        ];

        super::verify_no_collissions(&msgs, &states, &1);
    }

    #[test]
    #[should_panic]
    fn verify_collissions() {
        let msgs: [&[&str]; 4] = [&[], &["msg_b", "msg_c"], &["msg_a"], &["msg_d", "msg_a"]];
        let states = [
            State::Empty,
            State::Ongoing(1),
            State::Finished(0),
            State::Ongoing(1),
        ];

        super::verify_no_collissions(&msgs, &states, &3);
    }

    #[test]
    fn no_intersection() {
        let msgs: [&[&str]; 5] = [
            &["msg_b", "msg_c"],
            &["msg_d", "msg_e", "msg_f"],
            &["msg_a"],
            &["msg_g", "msg_h", "msg_i", "msg_j"],
            &[],
        ];

        assert_no_intersection(msgs);
    }

    #[test]
    #[should_panic]
    fn intersection() {
        let msgs: [&[&str]; 5] = [
            &["msg_b", "msg_c", "msg_i"],
            &["msg_d", "msg_e", "msg_f"],
            &["msg_a"],
            &["msg_g", "msg_h", "msg_i", "msg_j"],
            &[],
        ];

        assert_no_intersection(msgs);
    }

    #[test]
    fn single_interface_with_no_contract_msgs() {
        let msgs: [&[&str]; 2] = [&["msg_a", "msg_b"], &[]];

        assert_no_intersection(msgs);
    }
}
