//use ec::{EcMode, EcSys};

use std::{fs::OpenOptions, io::Read as _};

fn main() {
    // let mut ec = EcSys::new(EcMode::Read).unwrap();
    // let one = ec.ec_dump_pretty().unwrap();

    // println!("{one}");

    const EC_IO: &str = "/sys/module/ec_sys/parameters/write_support";

    let res = OpenOptions::new()
        .read(true)
        .write(false)
        .append(false)
        .truncate(false)
        .create(false)
        .create_new(false)
        .open(EC_IO);

    let mut file = res.unwrap();

    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    let write_enabled = buf.trim() == "Y";
    println!("enabled: {write_enabled} : {buf:?}");
}
