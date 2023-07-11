use crate::constants;
use std::f64::consts::PI;

// 章动相关计算
fn nutation2(t: f64) -> (f64, f64) {
    let t2 = t * t;
    let b = &constants::NUTATION_B;
    let mut dl = 0.0;
    let mut de = 0.0;

    for i in (0..b.len()).step_by(5) {
        let c = b[i] + b[i + 1] * t + b[i + 2] * t2;

        let a = if i == 0 { -1.742 * t } else { 0.0 };

        dl += (b[i + 3] + a) * c.sin();
        de += b[i + 4] * c.cos();
    }

    (dl / 100.0 / constants::RAD, de / 100.0 / constants::RAD)
}

fn nutation_lon2(w: f64) -> f64 {
    let (x, _) = nutation2(w);
    x
}

fn gxc_sun_lon(t: f64) -> f64 {
    // 光行差计算
    let v = -0.043126 + 628.301955 * t - 0.000002732 * t.powi(2);
    let e = 0.016708634 - 0.000042037 * t - 0.0000001267 * t.powi(2);

    (-20.49552 * (1.0 + e * v.cos())) / constants::RAD
}

fn gxc_moon_lon(_t: f64) -> f64 {
    -3.4e-6
} //月球经度光行差

#[test]
fn test_nutation() {
    println!("{:?}", nutation2(0.08582968944889204))
}

// 计算地球速度
fn e_v(t: f64) -> f64 {
    let f = 628.307585 * t;
    628.332 +
        21.0 * (1.527 + f).sin() +
        0.44 * (1.48 + f * 2.0).sin() +
        0.129 * (5.82 + f).sin() * t +
        0.00055 * (4.21 + f) * t.powi(2)
}

// 计算月球速度
fn m_v(t: f64) -> f64 {
    let mut v = 8399.71 - 914.0 * f64::sin(0.7848 + 8328.691425 * t + 0.0001523 * t * t); //误差小于5%
    v -=
        179.0 * f64::sin(2.543 + 15542.7543 * t) + //误差小于0.3%
        160.0 * f64::sin(0.1874 + 7214.0629 * t) +
        62.0 * f64::sin(3.14 + 16657.3828 * t) +
        34.0 * f64::sin(4.827 + 16866.9323 * t) +
        22.0 * f64::sin(4.9 + 23871.4457 * t) +
        12.0 * f64::sin(2.59 + 14914.4523 * t) +
        7.0 * f64::sin(0.23 + 6585.7609 * t) +
        5.0 * f64::sin(0.9 + 25195.624 * t) +
        5.0 * f64::sin(2.32 - 7700.3895 * t) +
        5.0 * f64::sin(3.88 + 8956.9934 * t) +
        5.0 * f64::sin(0.49 + 7771.3771 * t);
    v
}

// M_Lon: function (t, n) {
//     return XL1_calc(0, t, n);
//   },
//月球经度计算,返回Date分点黄经,传入世纪数,n是项数比例
fn moon_lon(t: f64, n: i32) -> f64 {
    xl1_calc(0, t, n)
}

#[test]
fn test_moon_lon() {
    let _exp = -4658.341531724926;
    let t = -0.555026229290277;
    let mn = 10;
    let r = moon_lon(t, mn);
    println!("r={}, e={}", r, (r - _exp).abs());
    assert!((r - _exp).abs() < 1e-6);
}

fn moon_a_lon(t: f64, mn: i32, sn: i32) -> f64 {
    moon_lon(t, mn) + gxc_moon_lon(t) - (earth_lon(t, sn) + gxc_sun_lon(t) + PI)
}
#[test]
fn test_moon_a_lon() {
    let _exp = -4314.497431333761;
    let t = -0.555026229290277;
    let mn = 10;
    let sn = 3;
    let r = moon_a_lon(t, mn, sn);
    assert!((r - _exp).abs() < 1e-6);
    println!("{}", r)
}

pub fn moon_lon_t(w: f64) -> f64 {
    let mut t;
    let mut v = 8399.70911033384;
    t = (w - 3.81034) / v;
    t += (w - moon_lon(t, 3)) / v;
    v = m_v(t); //v的精度0.5%，详见原文
    t += (w - moon_lon(t, 20)) / v;
    t += (w - moon_lon(t, -1)) / v;
    t
}

pub fn moon_a_lon_t(w: f64) -> f64 {
    let mut t;
    let mut v = 7771.37714500204;
    t = (w + 1.08472) / v;
    t += (w - moon_a_lon(t, 3, 3)) / v;
    v = m_v(t) - e_v(t); //v的精度0.5%，详见原文
    t += (w - moon_a_lon(t, 20, 10)) / v;
    t += (w - moon_a_lon(t, -1, 60)) / v;
    t
}

pub fn moon_a_lon_t2(w: f64) -> f64 {
    let mut t: f64;
    let mut v = 7771.37714500204;
    t = (w + 1.08472) / v;
    let t2 = t * t;
    t -=
        (-0.00003309 * t2 +
            0.10976 * f64::cos(0.784758 + 8328.6914246 * t + 0.000152292 * t2) +
            0.02224 * f64::cos(0.1874 + 7214.0628654 * t - 0.00021848 * t2) -
            0.03342 * f64::cos(4.669257 + 628.307585 * t)) /
        v;
    let l =
        moon_lon(t, 20) -
        (4.8950632 +
            628.3319653318 * t +
            0.000005297 * t * t +
            0.0334166 * f64::cos(4.669257 + 628.307585 * t) +
            0.0002061 * f64::cos(2.67823 + 628.307585 * t) * t +
            0.000349 * f64::cos(4.6261 + 1256.61517 * t) -
            20.5 / constants::RAD);
    v =
        7771.38 -
        914.0 * f64::sin(0.7848 + 8328.691425 * t + 0.0001523 * t * t) -
        179.0 * f64::sin(2.543 + 15542.7543 * t) -
        160.0 * f64::sin(0.1874 + 7214.0629 * t);
    t += (w - l) / v;
    t
}

#[test]
fn test_moon_a_lon_t2() {
    let w = 1709.0264035528476;
    let _exp = 0.22004859021701112;
    let r = moon_a_lon_t2(w);
    println!("{}", r);
    assert!((r - _exp).abs() < 1e-6);
}

fn solor_a_lon(t: f64, n: i32) -> f64 {
    // 太阳视黄经
    // return this.E_Lon(t, n) + nutationLon2(t) + gxc_sunLon(t) + Math.PI; //注意，这里的章动计算很耗时
    earth_lon(t, n) + nutation_lon2(t) + gxc_sun_lon(t) + PI
}

pub fn solor_a_lon_t(w: f64) -> f64 {
    let mut t: f64;
    let mut v = 628.3319653318;

    t = (w - 1.75347 - PI) / v;
    v = e_v(t); // v的精度0.03%，详见原文
    t += (w - solor_a_lon(t, 10)) / v;
    v = e_v(t); // 再算一次v有助于提高精度,不算也可以
    t += (w - solor_a_lon(t, -1)) / v;

    t
}

#[test]
fn test_solor_a_lon_t() {
    println!("{:?}", solor_a_lon_t(57.59586531581288)) //0.08383759200490044
}

pub fn solor_a_lon_t2(w: f64) -> f64 {
    let mut t;
    let v = 628.3319653318;

    t = (w - 1.75347 - PI) / v;
    t -=
        (0.000005297 * t * t +
            0.0334166 * (4.669257 + 628.307585 * t).cos() +
            0.0002061 * (2.67823 + 628.307585 * t).cos() * t) /
        v;
    t +=
        (w -
            earth_lon(t, 8) -
            PI +
            (20.5 + 17.2 * (2.1824 - 33.75705 * t).sin()) / constants::RAD) /
        v;
    t
}

#[test]
fn test_solor_a_lon_t2() {
    println!("{:?}", solor_a_lon_t2(61.261056745000964))
}

fn earth_lon(t: f64, n: i32) -> f64 {
    eph_calc(0, 0, t, n)
}

fn eph_calc(xt: usize, zn: usize, t: f64, n: i32) -> f64 {
    let t = t / 10.0; // 转为儒略千年数
    let mut tn = 1.0;
    let mut v = 0.0;

    let f: &Vec<f64> = &constants::XL0[xt];
    let pn = zn * 6 + 1;
    let mut n1: i32;
    let mut n2: i32;
    let mut n0: i32;
    let mut n_: i32;
    let n_0 = (f[pn + 1] - f[pn]) as i32; // N0 序列总数

    for i in 0..6 {
        n1 = f[pn + i] as i32;
        n2 = f[pn + 1 + i] as i32;
        n0 = n2 - n1;

        if n0 == 0 {
            continue;
        }

        n_ = if n < 0 {
            n2 // 确定项数
        } else {
            let mut tmp_n =
                (((3.0 * (n as f64) * (n0 as f64)) / (n_0 as f64) + 0.5).floor() as i32) + n1;
            if i != 0 {
                tmp_n += 3;
            }
            if tmp_n > n2 {
                tmp_n = n2;
            }
            tmp_n
        };

        let mut c = 0.0;
        for j in (n1..n_).step_by(3) {
            c += f[j as usize] * (f[(j as usize) + 1] + t * f[(j as usize) + 2]).cos();
        }
        v += c * tn;
        tn *= t;
    }

    v /= f[0];

    if xt == 0 {
        // 地球
        let t2 = t * t;
        let t3 = t2 * t;

        match zn {
            0 => {
                v += (-0.0728 - 2.7702 * t - 1.1019 * t2 - 0.0996 * t3) / constants::RAD;
            }
            1 => {
                v += (0.0 + 0.0004 * t + 0.0004 * t2 - 0.0026 * t3) / constants::RAD;
            }
            2 => {
                v += (-0.002 + 0.0044 * t + 0.0213 * t2 - 0.025 * t3) / 1000000.0;
            }
            _ => {}
        }
    } else {
        // 其它行星
        let dv = constants::XL0_XZB[(xt - 1) * 3 + zn];
        match zn {
            0 => {
                v -= (3.0 * t) / constants::RAD;
            }
            2 => {
                v += dv / 1000000.0;
            }
            _ => {
                v += dv / constants::RAD;
            }
        }
    }

    v
}

#[test]
fn test_eph_calc() {
    println!("{:?}", eph_calc(0, 0, 0.07971959025665916, 8));
    println!("{:?}", eph_calc(2, 0, 0.07971959025665916, -1));
}

// 月亮星历法计算
fn xl1_calc(zn: usize, t: f64, n: i32) -> f64 {
    let ob = &constants::XL1[zn];
    let mut v = 0.0;
    let mut tn = 1.0;
    let mut t2 = t * t;
    let mut t3 = t2 * t;
    let mut t4 = t3 * t;
    let t5 = t4 * t;
    let tx = t - 10.0;

    if zn == 0 {
        v +=
            (3.81034409 + 8399.684730072 * t - 3.319e-5 * t2 + 3.11e-8 * t3 - 2.033e-10 * t4) *
            constants::RAD; //月球平黄经(弧度)
        v +=
            5028.792262 * t +
            1.1124406 * t2 +
            0.00007699 * t3 -
            0.000023479 * t4 -
            0.0000000178 * t5; //岁差(角秒)
        if tx > 0.0 {
            v += -0.866 + 1.43 * tx + 0.054 * tx * tx; //对公元3000年至公元5000年的拟合,最大误差小于10角秒
        }
    }
    t2 /= 1e4;
    t3 /= 1e8;
    t4 /= 1e8;

    let n = if n < 0 { ob[0].len() } else { (n as usize) * 6 };
    for (i, f) in ob.iter().enumerate() {
        let mut c = 0.0;
        let n = (((n as f64) * (f.len() as f64)) / (ob[0].len() as f64) + 0.5).floor() as usize;
        let n = if i == 0 { n } else { n + 6 };
        let n = if n >= f.len() { f.len() } else { n };
        for j in (0..n).step_by(6) {
            c +=
                f[j] *
                f64::cos(f[j + 1] + t * f[j + 2] + t2 * f[j + 3] + t3 * f[j + 4] + t4 * f[j + 5]);
        }
        v += c * tn;
        tn *= t;
    }

    if zn != 2 {
        v /= constants::RAD;
    }

    v
}
