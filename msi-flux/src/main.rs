use ec::Ec;

fn main() {
    let ec = Ec::new().unwrap();
    let one = ec.battery_mode().unwrap();

    println!("{one:?}");
}
