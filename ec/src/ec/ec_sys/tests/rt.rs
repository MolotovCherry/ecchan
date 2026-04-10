use super::*;

#[test]
fn test_cpu_rt_fan_speed() {
    let ec = get_ec();
    let speed = ec.cpu_rt_fan_speed().unwrap();
    assert_eq!(speed, 0x19);
    assert_read(&ec, 0x71);
    assert_unwritten(&ec);
}

#[test]
fn test_cpu_rt_temp() {
    let ec = get_ec();
    let temp = ec.cpu_rt_temp().unwrap();
    assert_eq!(temp, 0x2b);
    assert_read(&ec, 0x68);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_rt_fan_speed() {
    let ec = get_ec();
    let speed = ec.gpu_rt_fan_speed().unwrap();
    assert_eq!(speed, 0x1e);
    assert_read(&ec, 0x89);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_rt_temp() {
    let ec = get_ec();
    let temp = ec.gpu_rt_temp().unwrap();
    assert_eq!(temp, 0x2e);
    assert_read(&ec, 0x80);
    assert_unwritten(&ec);
}
