#![allow(clippy::unreadable_literal)]
use std::f64::consts::FRAC_2_PI;

fn horner(arr: &[f64], v: f64) -> f64 {
    let mut z = 0.0;
    for a in arr {
        z = v * z + a;
    }
    z
}

pub fn bessel0(x: f64) -> f64 {
    let x = x.abs();

    let b0_a1a = [
        -184.9052456,
        77392.33017,
        -11214424.18,
        651619640.7,
        -13362590354.0,
        57568490574.0,
    ];

    let b0_a2a = [
        1.0,
        267.8532712,
        59272.64853,
        9494680.718,
        1029532985.0,
        57568490411.0,
    ];

    let b0_a1b = [
        0.2093887211e-6,
        -0.2073370639e-5,
        0.2734510407e-4,
        -0.1098628627e-2,
        1.0,
    ];

    let b0_a2b = [
        -0.934935152e-7,
        0.7621095161e-6,
        -0.6911147651e-5,
        0.1430488765e-3,
        -0.1562499995e-1,
    ];

    let y = x * x;
    if x < 8.0 {
        let a1 = horner(&b0_a1a, y);
        let a2 = horner(&b0_a2a, y);
        a1 / a2
    } else {
        let xx = x - 0.785398164;
        let y = 64.0 / y;
        let a1 = horner(&b0_a1b, y);
        let a2 = horner(&b0_a2b, y);
        (FRAC_2_PI / x).sqrt() * (xx.cos() * a1 - xx.sin() * a2 * 8.0 / x)
    }
}
