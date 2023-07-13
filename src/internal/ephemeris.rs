use crate::internal::{ constants, math_utils::{ self, llr_conv, xyz2llr } };
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

#[test]
fn test_xl1_calc() {
    let zn = 2;
    let t = 0.07998631032113629;
    let n = -1;
    let exp_ = 402012.9303201109;
    let r = xl1_calc(zn, t, n);
    println!("{:?}", r);
    assert!((r - exp_).abs() < 1e-6);
}

// 行星位置相关计算

#[derive(Debug, Default)]
pub struct PlanetCoordinates {
    pub ecliptic_longitude: f64, // 黄经
    pub ecliptic_latitude: f64, // 黄纬

    pub apparent_longitude: f64, // 视黄经
    pub apparent_latitude: f64, // 视黄纬
    pub apparent_right_ascension: f64, //视赤经
    pub apparent_declination: f64, // 视赤纬度

    pub radius: f64, // 向径
    pub earth_distance: f64, // 地心距
    pub light_time_distance: f64, // 光行距
    pub station_ra: f64, // 站赤经
    pub station_dec: f64, // 站赤纬
    pub distance: f64, // 视距离
    pub azimuth: f64, // 方位角
    pub altitude: f64, // 高度角
    pub sidereal_time: (f64, f64), // 恒星时, 真、平
    pub body: CelestialBody, //星体
}

#[derive(Debug, Default, PartialEq)]
pub enum CelestialBody {
    #[default]
    Moon, // 月
    Sun, // 太阳
    Mercury, // 水星
    Venus, // 金星
    Mars, // 火星
    Jupiter, // 木星
    Saturn, //土星
    Uranus, // 天王
    Neptune, //海王
    Pluto, // 冥王
}

// nutation2(T)
// hcjj(T)   -> obliquity //黄赤角
// pGST2(jd)  done
// e_coord(T, 15, 15, 15)  // done
// m_coord(T, 1, 1, -1) // done
// h2g(a, a2) //done
// llrConv(z, E) //done
// p_coord(0, T, -1, -1, -1) // done
// p_coord(xt, T, -1, -1, -1) // done
// rad2mrad(x) //done
// parallax(z, sj, fa, 0) //done
// MQC(x) // done
// rad2str(x, y)

/// 蒙气修正
// 大气折射（h 是真高度）
fn mqc(h: f64) -> f64 {
    0.0002967 / (h + 0.003138 / (h + 0.08919)).tan()
}

// 大气折射（ho 是视高度）
fn mqc2(ho: f64) -> f64 {
    -0.0002909 / (ho + 0.002227 / (ho + 0.07679)).tan()
}

/// 视差修正部分
// 视差修正
pub fn parallax(z: (f64, f64, f64), h: f64, fa: f64, high: f64) -> (f64, f64, f64) {
    let (a0, a1, mut a2) = z;
    let dw = if a2 < 500.0 { constants::CS_AU } else { 1.0 };
    a2 *= dw;
    let r0;
    let x0;
    let y0;
    let z0;
    let f = constants::CS_BA;
    let u = (f * fa.tan()).atan();
    let g = a0 + h;

    r0 = constants::CS_R_EAR * u.cos() + high * fa.cos(); // 站点与地地心向径的赤道投影长度
    z0 = constants::CS_R_EAR * u.sin() * f + high * fa.sin(); // 站点与地地心向径的轴向投影长度
    x0 = r0 * g.cos();
    y0 = r0 * g.sin();

    let (mut s0, mut s1, mut s2) = math_utils::llr2xyz((a0, a1, a2));

    s0 -= x0;
    s1 -= y0;
    s2 -= z0;
    let (s0, s1, s2) = xyz2llr((s0, s1, s2));

    (s0, s1, s2 / dw)
}

#[test]
fn test_parallax() {
    let z = (4.696514315023402, -0.29935945297735733, 32.35580180346149);
    let sj = -3.059632817522626;
    let fa = 0.38746309394274114;
    let hight = 0.0;
    let r = parallax(z, sj, fa, hight);
    let exp_ = (4.696514315023401, -0.29935945297735733, 32.35580180346149);
    println!("{:?} exp={:?}", r, exp_)
}

// 日心球面转地心球面
pub fn h2g(z: (f64, f64, f64), a: (f64, f64, f64)) -> (f64, f64, f64) {
    let (a0, a1, a2) = math_utils::llr2xyz(a);
    let (mut z0, mut z1, mut z2) = math_utils::llr2xyz(z);
    z0 -= a0;
    z1 -= a1;
    z2 -= a2;
    xyz2llr((z0, z1, z2))
}

// 黄赤角
pub fn obliquity(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    (84381.406 - 46.836769 * t - 0.0001831 * t2 + 0.0020034 * t3 - 5.76e-7 * t4 - 4.34e-8 * t5) /
        constants::RAD
}

//传入T是2000年首起算的日数(UT),dt是deltatT(日),精度要求不高时dt可取值为0
//返回格林尼治平恒星时(不含赤经章动及非多项式部分),即格林尼治子午圈的平春风点起算的赤经
fn pgst(t_: f64, dt: f64) -> f64 {
    let t = (t_ + dt) / 36525.0;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    // let rad = PI / 180.0;
    let pi2 = 2.0 * PI;
    pi2 * (0.779057273264 + 1.00273781191135448 * t_) +
        (0.014506 + 4612.15739966 * t + 1.39667721 * t2 - 0.00009344 * t3 + 0.00001882 * t4) /
            constants::RAD
}

//传入力学时J2000起算日数，返回平恒星时
pub fn pgst2(jd: f64) -> f64 {
    let dt = math_utils::dt_t(jd);
    pgst(jd - dt, dt)
}

///// 天体坐标计算

#[derive(Debug, Clone, Copy)]
pub enum PlutoParam {
    Pfi,
    Pw,
    PP,
    PQ,
    PE,
    Px,
    Ppi,
    PII,
    Pp,
    Pth,
    PZ,
    Pz,
}

#[derive(Debug, Clone, Copy)]
pub enum PlutoModel {
    IAU1976,
    IAU2000,
    P03,
}

/// 主要用于冥王星
pub fn prece(t: f64, sc: PlutoParam, model: PlutoModel) -> f64 {
    let (n, p) = match model {
        PlutoModel::IAU1976 => (4 as usize, &*constants::DIAU1976),
        PlutoModel::IAU2000 => (6 as usize, &*constants::DIAU2000),
        _ => (6 as usize, &*constants::DP03),
    };

    let mut c = 0_f64;

    let sc = sc as usize;
    let mut tn = 1.0;

    for i in 0..n {
        c += p[sc * n + i] * tn;
        tn *= t;
    }
    c / constants::RAD
}

#[test]
fn test_prece_p3() {
    let t = 0.07997720536404498;

    let exp_ = 0.0019535895398372087;
    let r = prece(t, PlutoParam::Pfi, PlutoModel::P03);
    assert!((r - exp_).abs() < 1e-6);

    let exp_2 = 0.4090925921852209;
    let r2 = prece(t, PlutoParam::Pw, PlutoModel::P03);
    assert!(r2 - exp_2 < 1e-6)
}

pub fn hdllr_j2d(t: f64, llr: (f64, f64, f64), model: PlutoModel) -> (f64, f64, f64) {
    let (mut r0, r1, r2) = llr;

    r0 += prece(t, PlutoParam::Pfi, model);
    let (mut r0, r1, r2) = llr_conv((r0, r1, r2), prece(t, PlutoParam::Pw, model));
    r0 -= prece(t, PlutoParam::Px, model);
    llr_conv((r0, r1, r2), -prece(t, PlutoParam::PE, model))
}

// 冥王行坐标计算
fn pluto_coord(t: f64) -> (f64, f64, f64) {
    let c0 = PI / 180.0 / 100000.0;
    let x = -1.0 + (2.0 * (t * 36525.0 + 1825394.5)) / 2185000.0;
    let t_ = t / 100000000.0;
    let mut r = [0.0, 0.0, 0.0];
    for i in 0..9 {
        let ob = &constants::XL0PLUTO[i];
        let n = ob.len();
        let mut v = 0.0;
        for j in (0..n).step_by(3) {
            v += ob[j] * (ob[j + 1] * t_ + ob[j + 2] * c0).sin();
        }
        if i % 3 == 1 {
            v *= x;
        }
        if i % 3 == 2 {
            v *= x * x;
        }
        r[i / 3] += v / 100000000.0;
    }
    r[0] += 9.922274 + 0.154154 * x;
    r[1] += 10.01609 + 0.064073 * x;
    r[2] += -3.947474 - 0.042746 * x;
    (r[0], r[1], r[2])
}

// 返回地球坐标
pub fn e_coord(t: f64, n1: i32, n2: i32, n3: i32) -> (f64, f64, f64) {
    let a = eph_calc(0, 0, t, n1);
    let b = eph_calc(0, 1, t, n2);
    let c = eph_calc(0, 2, t, n3);
    (a, b, c)
}

// 返回月球坐标
pub fn m_coord(t: f64, n1: i32, n2: i32, n3: i32) -> (f64, f64, f64) {
    let a = xl1_calc(0, t, n1);
    let b = xl1_calc(1, t, n2);
    let c = xl1_calc(2, t, n3);
    (a, b, c)
}

// 返回星体的坐标
pub fn p_coord(xt: usize, t: f64, n1: i32, n2: i32, n3: i32) -> (f64, f64, f64) {
    if xt < 8 {
        let a = eph_calc(xt, 0, t, n1);
        let b = eph_calc(xt, 1, t, n2);
        let c = eph_calc(xt, 2, t, n3);
        return (a, b, c);
    }

    // 太阳
    if xt == 9 {
        return (0.0, 0.0, 0.0);
    }
    // 冥王星
    let mut z = pluto_coord(t);
    z = xyz2llr(z);
    z = hdllr_j2d(t, z, PlutoModel::P03);
    z
}

#[test]
fn test_p_coord1() {
    //冥王星
    let xt = 8;
    let t = 0.08000458387191631;
    let r = p_coord(xt, t, -1, -1, -1);
    let exp_ = (4.691585897236764, 0.11304097852037004, 31.39534549865751);
    assert!((r.0 - exp_.0).abs() < 1e-6);
    assert!((r.1 - exp_.1).abs() < 1e-6);
    assert!((r.2 - exp_.2).abs() < 1e-6);
}

//// 待调试函数

// xt星体， jd 儒略日（相对于J2000天数），l经度， fa:纬度
pub fn xing_x(xt: usize, jd: f64, l: f64, fa: f64) -> String {
    //行星计算，jd力学时
    //基本参数计算
    let mut t = jd / 36525.0;
    let zd = nutation2(t);
    // ; let d_l = zd.0;
    // ; let d_e = zd.1; //章动
    let (d_l, d_e) = zd; //章动
    let e = obliquity(t) + d_e; //真黄赤交角
    let gst_ping = pgst2(jd); //平恒星时
    let gst = gst_ping + d_l * e.cos(); //真恒星时（不考虑非多项式部分）

    let mut z: (f64, f64, f64) = (0.0, 0.0, 0.0);
    let mut a: (f64, f64, f64);
    let mut z2: (f64, f64, f64);
    let mut a2: (f64, f64, f64);
    let mut s = String::new();
    let mut ra:f64;
    let mut rb:f64;
    let mut rc=0.0;

    if xt == 10 {
        //月亮
        //求光行时并精确求出地月距
        a = e_coord(t, 15, 15, 15); //地球
        z = m_coord(t, 1, 1, -1);
        ra = z.2; //月亮

        // let cs_agx = constants::CS_AU / constants::CS_GS / 86400.0 / 36525.0;
        t -= (ra * constants::CS_AGX) / constants::CS_AU; //光行时计算

        //求视坐标
        a2 = e_coord(t, 15, 15, 15); //地球
        z = m_coord(t, -1, -1, -1);
        rc = z.2; //月亮
        println!("z={:?}, a={:?}, ra={:?}", a, a2, ra);

        //求光行距
        a2 = h2g(a, a2);
        a2.2 *= constants::CS_AU;
        z2 = h2g(z, a2);
        rb = z2.2;

        //地心黄道及地心赤道
        z.0 = math_utils::rad2mrad(z.0 + d_l);
        s += &format!(
            "视黄经 {} 视黄纬 {} 地心距 {:.2}\n",

            math_utils::Angle::from_f64(z.0).degress(2), // rad2str(z[0], 0),
            math_utils::Angle::from_f64(z.1).degress(2), //rad2str(z[1], 0),
            ra
        );
        z = llr_conv(z, e); //转到赤道坐标
        s += &format!(
            "视赤经 {} 视赤纬 {} 光行距 {:.2}\n",
            math_utils::Angle::from_f64(z.0).time(2), //rad2str(z[0], 1),
            math_utils::Angle::from_f64(z.1).degress(2), //rad2str(z[1], 0),
            rb
        );
    }
    if xt < 10 {
        //行星和太阳
        a = p_coord(0, t, -1, -1, -1); //地球
        z = p_coord(xt, t, -1, -1, -1); //行星
        z.0 = math_utils::rad2mrad(z.0);
        s += &format!(
            "黄经一 {} 黄纬一 {} 向径一 {:.2}\n",
            math_utils::Angle::from_f64(z.0).degress(2), //rad2str(z[0], 0),
            math_utils::Angle::from_f64(z.1).degress(2), //rad2str(z[1], 0),
            z.2
        );

        //地心黄道
        z = h2g(z, a);
        ra = z.2; //ra地心距
        // let cs_agx = constants::CS_AU / constants::CS_GS / 86400.0 / 36525.0;
        t -= ra * constants::CS_AGX; // cs_agx; //光行时

        //重算坐标
        a2 = p_coord(0, t, -1, -1, -1); //地球
        z2 = p_coord(xt, t, -1, -1, -1); //行星
        z = h2g(z2, a);
        rb = z.2; //rb光行距（在惯性系中看）
        z = h2g(z2, a2);
        rc = z.2; //rc视距
        z.0 = math_utils::rad2mrad(z.0 + d_l); //补章动

        s += &format!(
            "视黄经 {} 视黄纬 {} 地心距 {:.2}\n",
            math_utils::Angle::from_f64(z.0).degress(2), //rad2str(z[0], 0),
            math_utils::Angle::from_f64(z.1).degress(2), //rad2str(z[1], 0),
            ra
        );
        z = llr_conv(z, e); //转到赤道坐标
        s += &format!(
            "视赤经 {} 视赤纬 {} 光行距 {:.2}\n",
            math_utils::Angle::from_f64(z.0).time(2), //rad2str(z[0], 1),
            math_utils::Angle::from_f64(z.1).degress(2), //rad2str(z[1], 0),
            rb
        );
    }

    let sj = math_utils::rad2rrad(gst + l - z.0); //得到天体时角

    z = parallax(z, sj, fa, 0.0); //视差修正
    s += &format!(
        "站赤经 {} 站赤纬 {} 视距离 {:.2}\n",
        math_utils::Angle::from_f64(z.0).time(2), //rad2str(z[0], 1),
        math_utils::Angle::from_f64(z.1).degress(2), //rad2str(z[1], 0),
        rc
    );

    z.0 += PI / 2.0 - gst - l; //修正了视差的赤道坐标
    z = llr_conv(z, PI / 2.0 - fa); //转到时角坐标转到地平坐标
    z.0 = math_utils::rad2mrad(PI / 2.0 - z.0);

    if z.1 > 0.0 {
        z.1 += mqc(z.1); //大气折射修正
    }
    s += &format!(
        "方位角 {} 高度角 {}\n",
        math_utils::Angle::from_f64(z.0).degress(2), //rad2str(z[0], 0),
        math_utils::Angle::from_f64(z.1).degress(2) // rad2str(z[1], 0)
    );
    s += &format!(
        "恒星时 {}(平) {}(真)\n",
        math_utils::Angle::from_f64(gst_ping).time(2), //,rad2str(rad2mrad(gst_ping), 1),
        math_utils::Angle::from_f64(gst).time(2) //(rad2mrad(gst), 1)
    );

    s
}

#[test]
fn test_xing_x() {
    let xt = 8_usize;
    let jd = 2921.167425921743;
    let l = 1.9911297824990999;
    let fa = 0.38746309394274114;
    println!("{}", xing_x(xt, jd, l, fa))
}

#[test]
fn test_xing_monn() {
    let xt = 10_usize;
    let jd = 2921.5;
    let l = 1.9911297824990999;
    let fa = -0.38746309394274114;
    println!("{}", xing_x(xt, jd, l, fa));
}
