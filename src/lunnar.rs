#[allow(dead_code)]
use std::f64::consts::PI;
use crate::math_utils;
use crate::ephemeris;
use crate::constants;

#[derive(Debug, Default)]
pub struct YearCalender {
    pub year: i32,
    pub zq: [f64; 25], // 中气的儒略日（相对于J2000）
    pub hs: [f64; 15], // 合朔的儒略日（相对于J2000）
    pub pe1: f64, //
    pub pe2: f64,

    pub dx: [f64; 15], // 记录月的天数
    pub ym: [String; 15], // 记录月名
    pub leap: i32, // 记录闰月月序
    pub lunar_month: [i32; 15], // 记录月份名， 0 表示十一月， 1 表示十二月 3表示正月  ;闰月 leap=1, lunar_month= leap<<8 | month
}

impl YearCalender {
    pub fn new(year: i32) -> Self {
        Self { year: year, zq: [0.0; 25], hs: [0.0; 15], dx: [0.0; 15], ..Default::default() }
    }
}

// jd: 相对于儒略日的天数,相对于J2000
pub fn calc_year_calendar(jd: f64) -> YearCalender {
    let j2000 = constants::J2000;
    let mut year_info = YearCalender::new(AstroyDate::jd2day(jd + j2000).0);

    let a: &mut [f64; 25] = &mut year_info.zq;
    let b = &mut year_info.hs;
    let mut w: f64;

    // 该年的气
    w = ((jd - 355.0 + 183.0) / 365.2422).floor() * 365.2422 + 355.0;
    if calc(w, "气") > jd {
        w -= 365.2422;
    }

    for i in 0..25 {
        a[i] = calc(w + 15.2184 * (i as f64), "气");
    }
    year_info.pe1 = calc(w - 15.2, "气");
    year_info.pe2 = calc(w - 30.4, "气");

    // 今年"首朔"的日月黄经差w
    w = calc(a[0], "朔");
    if w > a[0] {
        w -= 29.53;
    }

    // 该年所有朔,包含14个月的始末
    for i in 0..15 {
        b[i] = calc(w + 29.5306 * (i as f64), "朔");
    }

    // 月大小
    // *leap = 0;
    for i in 0..14 {
        year_info.dx[i] = b[i + 1] - b[i];
        year_info.ym[i] = i.to_string();
        year_info.lunar_month[i] = i as i32;
    }

    // -721年至-104年的后九月及月建问题,与朔有关，与气无关
    let yy = (((a[0] + 10.0 + 180.0) / 365.2422).floor() as i32) + 2000;
    if yy >= -721 && yy <= -104 {
        let mut ns: Vec<f64> = vec![0.0; 9];
        for i in 0..3 {
            let y = yy + i - 1;
            if y >= -721 {
                let jd1 =
                    1457698.0 -
                    j2000 +
                    ((0.342 + ((y + 721) as f64) * 12.368422) * 29.5306).floor();
                ns[i as usize] = calc(jd1, "朔");
                ns[(i + 3) as usize] = 13.0;
                ns[(i + 6) as usize] = 2.0;
            }
            if y >= -479 {
                let jd1 =
                    1546083.0 - j2000 + ((0.5 + ((y + 479) as f64) * 12.368422) * 29.5306).floor();
                ns[i as usize] = calc(jd1, "朔");
                ns[(i + 3) as usize] = 13.0;
                ns[(i + 6) as usize] = 2.0;
            }
            if y >= -220 {
                let jd1 =
                    1640641.0 - j2000 + ((0.866 + ((y + 220) as f64) * 12.369) * 29.5306).floor();
                ns[i as usize] = calc(jd1, "朔");
                ns[(i + 3) as usize] = 9.0;
                ns[(i + 6) as usize] = 11.0;
            }
        }
        for i in 0..14 {
            let mut nn = 2_i32;
            while nn >= 0 && year_info.hs[i as usize] < ns[nn as usize] {
                nn -= 1;
            }
            let f1 = ((year_info.hs[i] - ns[nn as usize] + 15.0) / 29.5306).floor() as i32;
            if f1 < 12 {
                year_info.ym[i] =
                    constants::YMC[
                        ((f1 + (ns[(nn + 6) as usize] as i32)) as usize) % 12
                    ].to_string();
            } else {
                year_info.ym[i] = ns[(nn + 3) as usize].to_string();
            }
        }
        return year_info;
    }

    if b[13] <= a[24] {
        //第13月的月末没有超过冬至(不含冬至),说明今年含有13个月
        let mut i = 1;
        while i < 13 && b[(i + 1) as usize] > a[2 * (i as usize)] {
            i += 1;
        }
        year_info.leap = i;

        while i < 14 {
            year_info.lunar_month[i as usize] = i - 1;
            i += 1;
        }
        let leap = year_info.leap;
        year_info.lunar_month[leap as usize] = (1 << 8) | year_info.lunar_month[leap as usize];
    }

    // 转为月建名
    for i in 0..14 {
        let is_leap = year_info.lunar_month[i] >> 8;
        let lm = year_info.lunar_month[i] & 0xff;
        if is_leap == 1 {
            year_info.ym[i as usize] = format!("闰{}", constants::YMC[(lm % 12) as usize]);
        } else {
            year_info.ym[i as usize] = format!("{}", constants::YMC[(lm % 12) as usize]);
        }
    }

    year_info
}

#[test]
fn test_convert_to_lunar() {
    // 算公历对应的农历
    let y = 2023;
    let m = 11;
    let d = 19;
    let jd2 = AstroyDate::from_day(y, m, (d as f64) + 0.5).jd - constants::J2000;

    let yi = calc_year_calendar(jd2);
    println!("{:?}", yi.hs);

    // let mut x=0;
    let mut i = 0;
    println!("jd2={}", jd2);

    while jd2 > yi.hs[i] {
        i += 1;
    }

    println!(
        "公历：{}年{}月{}日  农历:{}年 {}月 {}日",
        y,
        m,
        d,
        yi.year,
        yi.ym[i - 1],
        (jd2 - yi.hs[i - 1] + 1.0) as i32
    );
}

#[test]
fn test_calc_year_calendar() {
    let jd = 545.0;

    // [330, 360, 389, 419, 449, 478, 508, 537, 567, 596, 625, 655, 684, 714, 743]
    // [355, 370, 385, 400, 414, 429, 444, 460, 475, 490, 506, 521, 537, 553, 569, 584, 600, 615, 631, 646, 661, 676, 691, 706, 721, pe1: 341, pe2: 326]

    let r = calc_year_calendar(jd);
    println!("{:?}", r);
}

#[derive(Default, Debug)]
pub struct AstroyDate {
    pub jd: f64, // 儒略日
}
impl AstroyDate {
    pub fn new(x: f64) -> Self {
        Self {
            jd: x,
            ..Default::default()
        }
    }

    pub fn jd2day(x: f64) -> (i32, i32, f64) {
        //    var r=new Object();
        //    var D=int2(jd+0.5), F=jd+0.5-D, c;  //取得日数的整数部份A及小数部分F
        //    if(D>=2299161) c=int2((D-1867216.25)/36524.25),D+=1+c-int2(c/4);
        //    D += 1524;               r.Y = int2((D-122.1)/365.25);//年数
        //    D -= int2(365.25*r.Y);   r.M = int2(D/30.601); //月数
        //    D -= int2(30.601*r.M);   r.D = D; //日数
        //    if(r.M>13) r.M -= 13, r.Y -= 4715;
        //    else       r.M -= 1,  r.Y -= 4716;
        //    //日的小数转为时分秒
        //    F*=24; r.h=int2(F); F-=r.h;
        //    F*=60; r.m=int2(F); F-=r.m;
        //    F*=60; r.s=F;
        let mut year: i32;
        let mut month: i32;
        let day: f64;
        let c: i32;
        let mut d = (x + 0.5).floor() as i32;
        let r = x + 0.5 - (d as f64);
        if d >= 2299161 {
            c = (((d as f64) - 1867216.25) / 36524.25).floor() as i32;
            d += 1 + c - c / 4;
        }
        d += 1524;
        year = (((d as f64) - 122.1) / 365.25).floor() as i32;

        d -= (365.25 * (year as f64)).floor() as i32;
        month = ((d as f64) / 30.601).floor() as i32;
        d -= (30.601 * (month as f64)).floor() as i32;
        if month > 13 {
            month -= 13;
            year -= 4715;
        } else {
            month -= 1;
            year -= 4716;
        }
        day = (d as f64) + r;
        (year, month, day)
    }

    pub fn from_day(mut y: i32, mut m: i32, d: f64) -> Self {
        let mut n = 0;
        let mut is_green = false; // 判断是否是格林历法
        if y * 372 + m * 31 + (d.floor() as i32) >= 588829 {
            is_green = true;
        }
        if m <= 2 {
            m += 12;
            y -= 1;
        }

        if is_green {
            n = y / 100;
            n = 2 - n + n / 4;
        }
        // int2(365.25*(y+4716)) + int2(30.6001*(m+1))+d+n - 1524.5;

        let _jd =
            (365.25 * ((y + 4716) as f64)).floor() +
            (30.6001 * ((m + 1) as f64)).floor() +
            d +
            (n as f64) -
            1524.5;

        Self { jd: _jd }
    }
}

#[test]
fn test_astoy() {
    let d = AstroyDate::from_day(2023, 6, 29.5);
    println!("{:?}", d.jd);
    println!("{:?}", AstroyDate::new(2460125.0));
    println!("{:?}", AstroyDate::jd2day(2460125.0))
}

pub fn calc(jd: f64, qs: &str) -> f64 {
    let jd = jd + 2451545.0;
    let mut i;
    let mut d: i32;
    let n;
    let mut b = constants::SUO_KB.to_vec();
    let mut pc = 14;
    if qs == "气" {
        b = constants::QI_KB.to_vec();
        pc = 7;
    }
    let f1 = b[0] - (pc as f64);
    let f2 = b[b.len() - 1] - (pc as f64);
    let f3 = 2436935.0;

    if jd < f1 || jd >= f3 {
        if qs == "气" {
            return (
                qi_hight(
                    ((((jd + (pc as f64) - 2451259.0) / 365.2422) * 24.0).floor() * PI) / 12.0
                ) + 0.5
            ).floor();
        } else {
            return (
                so_high(((jd + (pc as f64) - 2451551.0) / 29.5306).floor() * PI * 2.0) + 0.5
            ).floor();
        }
    }

    if jd >= f1 && jd < f2 {
        i = 0;
        while i < b.len() {
            if jd + (pc as f64) < b[i + 2] {
                break;
            }
            i += 2;
        }
        d = (b[i] + b[i + 1] * ((jd + (pc as f64) - b[i]) / b[i + 1] + 0.5).floor()) as i32;
        if d == 1683460 {
            d += 1;
        }
        return (d - 2451545) as f64;
    }

    if jd >= f2 && jd < f3 {
        if qs == "气" {
            let qb = &*constants::QB;
            d = (
                qi_low(((((jd + (pc as f64) - 2451259.0) / 365.2422) * 24.0).floor() * PI) / 12.0) +
                0.5
            ).floor() as i32;
            println!("d={}", d);
            n = qb
                .chars()
                .nth((((jd - f2) / 365.2422) * 24.0).floor() as usize)
                .unwrap()
                .to_string(); //找定气修正值
            println!("n={}", n);
        } else {
            let sb = &*constants::SB;
            d = (
                so_low(((jd + (pc as f64) - 2451551.0) / 29.5306).floor() * PI * 2.0).floor() + 0.5
            ).floor() as i32;
            n = sb
                .chars()
                .nth(((jd - f2) / 29.5306).floor() as usize)
                .unwrap()
                .to_string(); //找定朔修正值
        }
        if n == "1" {
            return (d + 1) as f64;
        } else if n == "2" {
            return (d - 1) as f64;
        } else {
            return d as f64;
        }
    }
    jd
}

#[test]
fn test_calc_qi() {
    let jd = -20463.8054;
    let qs = "气";
    let _exp = -20463.0;
    let r = calc(jd, qs);
    println!("exp={}, r={}", _exp, r);
    // assert!((r-exp).abs() as f64<1e-6);
}

#[test]
fn test_calc_shuo() {
    let jd = -20463.0;
    let qs = "朔";
    let _exp = -20459.0;
    let r = calc(jd, qs);
    println!("exp={}, r={}", _exp, r);
}


// 朔计算
fn so_low(w: f64) -> f64 {
    let v = 7771.37714500204;
    let mut t = (w + 1.08472) / v;
    t -=
        (-0.0000331 * t * t +
            0.10976 * f64::cos(0.785 + 8328.6914 * t) +
            0.02224 * f64::cos(0.187 + 7214.0629 * t) -
            0.03342 * f64::cos(4.669 + 628.3076 * t)) /
            v +
        (32.0 * (t + 1.8) * (t + 1.8) - 20.0) / 86400.0 / 36525.0;
    t * 36525.0 + 8.0 / 24.0
}

pub fn so_high(w: f64) -> f64 {
    let t = ephemeris::moon_a_lon_t2(w) * 36525.0;
    let t = t - math_utils::dt_t(t) + 8.0 / 24.0;
    let v = ((t + 0.5) % 1.0) * 86400.0;
    let mut result = t;
    if v < 1800.0 || v > 86400.0 - 1800.0 {
        result = ephemeris::moon_a_lon_t(w) * 36525.0 - math_utils::dt_t(t) + 8.0 / 24.0;
    }
    result
}

pub fn so_accurate(w: f64) -> f64 {
    let t = ephemeris::moon_a_lon_t(w) * 36525.0;
    t - math_utils::dt_t(t) + 8.0 / 24.0
}

pub fn so_accurate2(jd: f64) -> f64 {
    so_accurate(((jd + 8.0) / 29.5306).floor() * PI * 2.0)
}

#[test]
fn test_so_accurate2() {
    let jd = 330.0;
    let _exp = 329.79951510797684;
    let r = so_accurate2(jd);
    println!("exp={}, r={}", _exp, r);
    assert!((r - _exp).abs() < 1e-6);
}

#[test]
fn test_so() {
    let h_w = 1727.8759594743863;
    let h_r = 8126.101574259753;
    println!("exp={}, r={}", h_r, so_high(h_w));
    assert!((so_high(h_w) - h_r).abs() < 1e-6);
    let l_w = -4354.247417875453;
    let l_r = -20458.974805811675;
    println!("exp={}, r={}", l_r, so_low(l_w));
    assert!((so_low(l_w) - l_r).abs() < 1e-6);
}

fn qi_low(w: f64) -> f64 {
    let mut t = (w - 4.895062166) / 628.3319653318; // 第一次估算
    t -=
        (53.0 * t * t +
            334116.0 * (4.67 + 628.307585 * t).cos() +
            2061.0 * (2.678 + 628.3076 * t).cos() * t) /
        (628.3319653318 * 10000000.0); // 第二次估算

    let l =
        48950621.66 +
        6283319653.318 * t +
        53.0 * t * t + // 平黄经
        334166.0 * (4.669257 + 628.307585 * t).cos() + // 地球椭圆轨道级数展开
        3489.0 * (4.6261 + 1256.61517 * t).cos() + // 地球椭圆轨道级数展开
        2060.6 * (2.67823 + 628.307585 * t).cos() * t - // 一次泊松项
        994.0 -
        834.0 * (2.1824 - 33.75705 * t).sin(); // 光行差与章动修正

    t -=
        (l / 10000000.0 - w) / 628.332 +
        (32.0 * (t + 1.8) * (t + 1.8) - 20.0) / (86400.0 * 36525.0);
    t * 36525.0 + 8.0 / 24.0
}

#[test]
fn test_qi_low() {
    let w = -1949.3582415524668;
    let r = -113600.34639807165;
    assert!((qi_low(w) - r).abs() < 1e-6);
    println!("{}", qi_low(w));
}

fn qi_hight(w: f64) -> f64 {
    let mut t = ephemeris::solor_a_lon_t2(w) * 36525.0;
    t = t - math_utils::dt_t(t) + 8.0 / 24.0;
    let v = ((t + 0.5) % 1.0) * 86400.0;
    if v < 1200.0 || v > 86400.0 - 1200.0 {
        t = ephemeris::solor_a_lon_t(w) * 36525.0 - math_utils::dt_t(t) + 8.0 / 24.0;
    }
    t
}

#[test]
fn test_qi_hight() {
    println!("{:?}", qi_hight(58.119464091411174)); // 3093.8331491526683
    assert!((qi_hight(58.119464091411174) - 3093.8331491526683).abs() < 1e-6)
}

pub fn qi_accurate(w: f64) -> f64 {
    let t = ephemeris::solor_a_lon_t(w) * 36525.0;
    t - math_utils::dt_t(t) + 8.0 / 24.0
}

pub fn qi_accurate2(jd: f64) -> f64 {
    let d = PI / 12.0;
    let w = (((jd + 293.0) / 365.2422) * 24.0).floor() * d;
    let a = qi_accurate(w);
    if a - jd > 5.0 {
        return qi_accurate(w - d);
    } else if a - jd < -5.0 {
        return qi_accurate(w + d);
    } else {
        return a;
    }
}

#[test]
fn test_qi_accurrate2() {
    let jd = 341.0;
    let _exp = 340.65072186631636;
    let r = qi_accurate2(jd);
    println!("exp={}, r={}, r-exp={}", _exp, r, (r - _exp) * (10.0_f64).powi(6));
    assert!((r - _exp).abs() < 1e-6)
}
