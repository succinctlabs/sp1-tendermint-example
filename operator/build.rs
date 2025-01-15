use sp1_helper::{build_program_with_args, BuildArgs};

fn main() {
    build_program_with_args(
        "../program",
        BuildArgs {
            elf_name: Some("tendermint-light-client".to_string()),
            ..Default::default()
        },
    )
}
