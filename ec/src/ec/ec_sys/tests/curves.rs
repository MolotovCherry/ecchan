use super::*;

//
// Fan Curve
//

#[test]
fn test_cpu_fan_curve() {
    let ec = get_ec();
    let curve = ec.cpu_fan_curve().unwrap();
    assert_eq!(
        Curve7 {
            n1: 25,
            n2: 25,
            n3: 35,
            n4: 55,
            n5: 65,
            n6: 70,
            n7: 80
        },
        curve
    );

    assert_read_range(&ec, 0x72..=0x78);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_fan_curve() {
    let ec = get_ec();
    let curve = ec.gpu_fan_curve().unwrap();

    assert_eq!(
        Curve7 {
            n1: 0,
            n2: 30,
            n3: 40,
            n4: 55,
            n5: 60,
            n6: 70,
            n7: 80
        },
        curve
    );

    assert_read_range(&ec, 0x8A..=0x90);
    assert_unwritten(&ec);
}

#[test]
fn test_set_cpu_fan_curve() {
    let mut ec = get_ec();

    let curve = Curve7 {
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        n6: 6,
        n7: 7,
    };

    ec.set_cpu_fan_curve(curve).unwrap();
    assert_write_range(&ec, 0x72..=0x78, &[1, 2, 3, 4, 5, 6, 7]);
    assert_unread(&ec);
}

#[test]
fn test_set_gpu_fan_curve() {
    let mut ec = get_ec();

    let curve = Curve7 {
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        n6: 6,
        n7: 7,
    };

    ec.set_gpu_fan_curve(curve).unwrap();
    assert_write_range(&ec, 0x8A..=0x90, &[1, 2, 3, 4, 5, 6, 7]);
    assert_unread(&ec);
}

//
// Temp curve
//

#[test]
fn test_cpu_temp_curve() {
    let ec = get_ec();
    let curve = ec.cpu_temp_curve().unwrap();
    assert_eq!(
        Curve7 {
            n1: 0,
            n2: 55,
            n3: 64,
            n4: 70,
            n5: 76,
            n6: 82,
            n7: 88
        },
        curve
    );

    assert_read_range(&ec, 0x69..=0x6F);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_temp_curve() {
    let ec = get_ec();
    let curve = ec.gpu_temp_curve().unwrap();

    assert_eq!(
        Curve7 {
            n1: 0,
            n2: 52,
            n3: 58,
            n4: 64,
            n5: 70,
            n6: 76,
            n7: 82
        },
        curve
    );

    assert_read_range(&ec, 0x81..=0x87);
    assert_unwritten(&ec);
}

#[test]
fn test_set_cpu_temp_curve() {
    let mut ec = get_ec();

    let curve = Curve7 {
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        n6: 6,
        n7: 7,
    };

    ec.set_cpu_temp_curve(curve).unwrap();
    assert_write_range(&ec, 0x69..=0x6F, &[1, 2, 3, 4, 5, 6, 7]);
    assert_unread(&ec);
}

#[test]
fn test_set_gpu_temp_curve() {
    let mut ec = get_ec();

    let curve = Curve7 {
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        n6: 6,
        n7: 7,
    };

    ec.set_gpu_temp_curve(curve).unwrap();
    assert_write_range(&ec, 0x81..=0x87, &[1, 2, 3, 4, 5, 6, 7]);
    assert_unread(&ec);
}

//
// Hysteresis Curve
//

#[test]
fn test_cpu_hysteresis_curve() {
    let ec = get_ec();
    let curve = ec.cpu_hysteresis_curve().unwrap();
    assert_eq!(
        Curve6 {
            n1: 10,
            n2: 3,
            n3: 3,
            n4: 3,
            n5: 3,
            n6: 3,
        },
        curve
    );

    assert_read_range(&ec, 0x7A..=0x7F);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_hysteresis_curve() {
    let ec = get_ec();
    let curve = ec.gpu_hysteresis_curve().unwrap();

    assert_eq!(
        Curve6 {
            n1: 7,
            n2: 3,
            n3: 3,
            n4: 3,
            n5: 3,
            n6: 3,
        },
        curve
    );

    assert_read_range(&ec, 0x92..=0x97);
    assert_unwritten(&ec);
}

#[test]
fn test_set_cpu_hysteresis_curve() {
    let mut ec = get_ec();

    let curve = Curve6 {
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        n6: 6,
    };

    ec.set_cpu_hysteresis_curve(curve).unwrap();
    assert_write_range(&ec, 0x7A..=0x7F, &[1, 2, 3, 4, 5, 6]);
    assert_unread(&ec);
}

#[test]
fn test_set_gpu_hysteresis_curve() {
    let mut ec = get_ec();

    let curve = Curve6 {
        n1: 1,
        n2: 2,
        n3: 3,
        n4: 4,
        n5: 5,
        n6: 6,
    };

    ec.set_gpu_hysteresis_curve(curve).unwrap();
    assert_write_range(&ec, 0x92..=0x97, &[1, 2, 3, 4, 5, 6]);
    assert_unread(&ec);
}

//
//
//

#[test]
#[should_panic = "no dgpu available"]
fn test_gpu_fan_curve_unsupported() {
    let ec = get_ec();
    no_dgpu();
    ec.gpu_fan_curve().unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_gpu_temp_curve_unsupported() {
    let ec = get_ec();
    no_dgpu();
    ec.gpu_temp_curve().unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_gpu_hysteresis_curve_unsupported() {
    let ec = get_ec();
    no_dgpu();
    ec.gpu_hysteresis_curve().unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_set_gpu_fan_curve_unsupported() {
    let mut ec = get_ec();
    no_dgpu();
    ec.set_gpu_fan_curve(Default::default()).unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_set_gpu_temp_curve_unsupported() {
    let mut ec = get_ec();
    no_dgpu();
    ec.set_gpu_temp_curve(Default::default()).unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_set_gpu_hysteresis_curve_unsupported() {
    let mut ec = get_ec();
    no_dgpu();
    ec.set_gpu_hysteresis_curve(Default::default()).unwrap();
}

#[test]
#[should_panic = "only wmi2 is supported"]
fn test_gpu_fan_curve_wmi_unsupported() {
    let mut ec = get_ec();
    ec.sys.as_mut().unwrap().1.ver = WmiVer::Wmi1;
    ec.gpu_fan_curve().unwrap();
}

#[test]
#[should_panic = "only wmi2 is supported"]
fn test_gpu_temp_curve_wmi_unsupported() {
    let mut ec = get_ec();
    ec.sys.as_mut().unwrap().1.ver = WmiVer::Wmi1;
    ec.gpu_temp_curve().unwrap();
}

#[test]
#[should_panic = "only wmi2 is supported"]
fn test_gpu_hysteresis_curve_wmi_unsupported() {
    let mut ec = get_ec();
    ec.sys.as_mut().unwrap().1.ver = WmiVer::Wmi1;
    ec.gpu_hysteresis_curve().unwrap();
}

#[test]
#[should_panic = "only wmi2 is supported"]
fn test_set_gpu_fan_curve_wmi_unsupported() {
    let mut ec = get_ec();
    ec.sys.as_mut().unwrap().1.ver = WmiVer::Wmi1;
    ec.set_gpu_fan_curve(Default::default()).unwrap();
}

#[test]
#[should_panic = "only wmi2 is supported"]
fn test_set_gpu_temp_curve_wmi_unsupported() {
    let mut ec = get_ec();
    ec.sys.as_mut().unwrap().1.ver = WmiVer::Wmi1;
    ec.set_gpu_temp_curve(Default::default()).unwrap();
}

#[test]
#[should_panic = "only wmi2 is supported"]
fn test_set_gpu_hysteresis_curve_wmi_unsupported() {
    let mut ec = get_ec();
    ec.sys.as_mut().unwrap().1.ver = WmiVer::Wmi1;
    ec.set_gpu_hysteresis_curve(Default::default()).unwrap();
}
