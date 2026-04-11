use ec::Ec;

fn main() {
    let mut ec = Ec::new().unwrap();
    let one = ec.ec_dump_pretty().unwrap();

    println!("{one}");
}
