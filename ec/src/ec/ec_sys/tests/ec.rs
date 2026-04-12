use super::*;

//
// Single read/write
//

#[test]
fn test_read_1() {
    let ec = get_ec();
    let io = get_io!(ec);
    let val = io.ec_read(0x2F).unwrap();
    assert_read(&ec, 0x2F);
    assert_eq!(0x5B, val);
    assert_unwritten(&ec);
}

#[test]
fn test_write_1() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);
    unsafe {
        io.ec_write(0x2F, 0xFB).unwrap();
    }

    assert_write(&ec, 0x2F, 0xFB);
    assert_unread(&ec);
}

//
// End Seq read/write
//

#[test]
fn test_end_seq_read_1() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write(0xFF, 0xFE).unwrap();
    }

    let mut buf = [0];
    io.ec_read_seq(0xFF..=0xFF, &mut buf).unwrap();
    assert_read(&ec, 0xFF);
    assert_eq!(0xFE, buf[0]);
    assert_write(&ec, 0xFF, 0xFE);
}

#[test]
#[should_panic = "buf len 2 must equal span of 0xFF..=0xFF"]
fn test_end_seq_read_2() {
    let ec = get_ec();
    let io = get_io!(ec);

    let mut buf = [0, 0];
    io.ec_read_seq(0xFF..=0xFF, &mut buf).unwrap();
}

#[test]
fn test_end_write_seq_1() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_seq(0xFF..=0xFF, &[0x44]).unwrap();
    }

    assert_write(&ec, 0xFF, 0x44);
    assert_unread(&ec);
}

#[test]
#[should_panic = "buf len 2 must equal span of 0xFF..=0xFF"]
fn test_end_seq_write_2() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_seq(0xFF..=0xFF, &[0xCE, 0xDE]).unwrap();
    }
}

//
// General seq read/write
//

#[test]
fn test_seq_read_4() {
    let ec = get_ec();
    let io = get_io!(ec);

    let mut buf = [0; 4];
    io.ec_read_seq(0xF2..=0xF5, &mut buf).unwrap();
    assert_read_range(&ec, 0xF2..=0xF5);
    assert_eq!([0x70, 0x00, 0x23, 0x44], buf);
    assert_unwritten(&ec);
}

#[test]
fn test_seq_write_4() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let vals = &[0x01, 0x02, 0x03, 0x04];

    unsafe {
        io.ec_write_seq(0xF2..=0xF5, vals).unwrap();
    }

    assert_write_range(&ec, 0xF2..=0xF5, vals);
    assert_unread(&ec);
}

//
// Bit tests
//

#[test]
fn test_read_bit() {
    let ec = get_ec();
    let io = get_io!(ec);

    let set = io.ec_read_bit(0x01, Bit::_6).unwrap();
    assert_read(&ec, 0x01);
    assert!(!set, "bit 6 not set");

    let set = io.ec_read_bit(0x01, Bit::_7).unwrap();
    assert_read(&ec, 0x01);
    assert!(set, "bit 7 set");

    assert_unwritten(&ec);
}

#[test]
fn test_write_bit() {
    let mut ec = get_ec();
    let io = get_io!(ec);

    let set = io.ec_read_bit(0x01, Bit::_6).unwrap();
    assert_read(&ec, 0x01);
    assert!(!set, "bit 6 not set");

    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_bit(0x01, Bit::_6, true).unwrap();
    }

    let set = io.ec_read_bit(0x01, Bit::_6).unwrap();
    assert_read(&ec, 0x01);
    assert!(set, "bit 6 set");

    assert_write(&ec, 0x01, 0xC0);

    drop(ec);

    //

    let mut ec = get_ec();
    let io = get_io!(ec);

    let set = io.ec_read_bit(0x01, Bit::_7).unwrap();
    assert_read(&ec, 0x01);
    assert!(set, "bit 7 set");

    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_bit(0x01, Bit::_7, false).unwrap();
    }

    let set = io.ec_read_bit(0x01, Bit::_7).unwrap();
    assert_read(&ec, 0x01);
    assert!(!set, "bit 7 not set");

    assert_write(&ec, 0x01, 0x00);
}
