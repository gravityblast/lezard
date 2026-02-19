use nssa_core::program::{
    AccountPostState, DEFAULT_PROGRAM_ID, ProgramInput, read_nssa_inputs, write_nssa_outputs,
};

// Double example program.
//
// Initializes an account's data field with 1 and
// then doubles the number every time it's executed

type Instruction = ();

fn main() {
    let (
        ProgramInput {
            pre_states,
            instruction: _,
        },
        instruction_data,
    ) = read_nssa_inputs::<Instruction>();

    let [pre_state] = pre_states
        .try_into()
        .unwrap_or_else(|_| panic!("Expected a single account"));

    let post_account = {
        let mut this = pre_state.account.clone();
        let bytes = this.data.into_inner();

        let value: u64 = if bytes.is_empty() {
            1
        } else {
            let arr: [u8; 8] = bytes
                .try_into()
                .unwrap_or_else(|_| panic!("Account data is not a valid u64"));
            u64::from_le_bytes(arr) * 2
        };

        this.data = value
            .to_le_bytes()
            .to_vec()
            .try_into()
            .expect("Data should fit within the allowed limits");
        this
    };

    let post_state = if post_account.program_owner == DEFAULT_PROGRAM_ID {
        AccountPostState::new_claimed(post_account)
    } else {
        AccountPostState::new(post_account)
    };

    write_nssa_outputs(instruction_data, vec![pre_state], vec![post_state]);
}
