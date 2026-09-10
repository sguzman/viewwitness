use viewwitness::{diff_to_yaml, diff_witnesses, from_yaml};

fn main() {
    let before = from_yaml(include_str!("transitions/03-busy-state/before.yaml"))
        .expect("before witness parses");
    let after = from_yaml(include_str!("transitions/03-busy-state/after.yaml"))
        .expect("after witness parses");

    let diff = diff_witnesses(&before, &after);
    print!("{}", diff_to_yaml(&diff).expect("diff serializes"));
}
