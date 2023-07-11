
#[allow(dead_code)]

use std::f64::consts::PI;
use crate::constants;

/**
* 角度定义
* rad  0~2PI
* mrad -pi - 2Pi
*/
#[derive(Default, Debug)]
pub struct Angle {
    pub rad: f64,
    pub mrad: f64,

    pub hours: Option<i32>,
    pub minutes: Option<i32>,
    pub seconds: Option<f64>,

    pub deg: Option<i32>,
    pub deg_m: Option<i32>,
    pub deg_s: Option<f64>,
}

impl Angle {
    /// 从浮点数构造度数
    pub fn from_f64(x: f64) -> Self {
        let mut a = x - (x / (PI * 2.0)).floor() * (PI * 2.0);
        if a < 0.0 {
            a = a + 2.0 * PI
        }
        let mut b = a;
        if b > PI {
            b = b - 2.0 * PI
        }
        Angle {
            rad: a,
            mrad: b,
            ..Default::default()
        }
    }

    /// 从角度构造度数
    pub fn from_degress(deg: &str) -> Self {
        let parts: Vec<&str> = deg.split(|c| c == '°' || c == '\'' || c == '\"').collect();
        let degress = parts[0].trim().parse::<i32>().unwrap();
        let minutes = parts[1].trim().parse::<i32>().unwrap();
        let seconds = parts[2].trim().parse::<f64>().unwrap();
        let f: f64 = (degress as f64) + (minutes as f64) / 60.0 + seconds / 60.0 / 60.0;
        let mut angle = Angle::from_f64(f * PI / 180.0);
        angle.deg = Some(degress);
        angle.deg_m = Some(minutes);
        angle.deg_s = Some(seconds);
        angle
    }

    pub fn f2tuple(mut cur: f64, ext: i32) -> (i32, i32, f64) {
        let base = 10.0_f64;
        let mut d = cur.floor() as i32;
        cur = (cur - (d as f64)) * 60.0;
        let mut m = cur.floor() as i32;
        cur = (cur - (m as f64)) * 60.0;
        let mut s = cur * base.powi(ext);
        s = s.round();
        s /= base.powi(ext);
        if s > 60.0 {
            s -= 1.0;
            m += 1;
        }
        if m > 60 {
            m -= 1;
            d += 1;
        }
        (d, m, s)
    }

    pub fn radis(&self) -> String {
        format!("radis:{:?}, mradis:{:?}", self.rad, self.mrad)
    }

    pub fn time(&mut self, ext: i32) -> String {
        match (self.hours, self.minutes, self.seconds) {
            (Some(x), Some(y), Some(z)) => {
                format!("{:?}h {:?}m {:?}s", x, y, z)
            }
            _ => {
                let cur = self.rad / (PI) * 12.0;
                let (d, m, s) = Self::f2tuple(cur, ext);
                (self.hours, self.minutes, self.seconds) = (Some(d), Some(m), Some(s));

                format!("{:?}h {:?}m {:?}s", d, m, s)
            }
        }
    }

    /// 输出度数
    ///
    pub fn degress(&mut self, ext: i32) -> String {
        match (self.deg, self.deg_m, self.deg_s) {
            (Some(x), Some(y), Some(z)) => {
                format!("{:?}° {:?}' {:?}\"", x, y, z)
            }
            _ => {
                let cur = self.rad / (PI) * 180.0;
                let (d, m, s) = Self::f2tuple(cur, ext);
                (self.deg, self.deg_m, self.deg_s) = (Some(d), Some(m), Some(s));
                format!("{:?}° {:?}' {:?}\"", d, m, s)
            }
        }
    }
}


#[test]
fn test_angle() {
    print!("to angle {:?}\n", Angle::from_f64(-11.24));
    let a = Angle::from_f64(-11.24).radis();
    println!("{}", &a);

    let mut c = Angle::from_f64(PI / 6.0);
    println!("{}", c.degress(2));

    let mut e = Angle::from_degress("30° 0' 0.0\"");

    println!("{}", e.radis());
    println!("{}", e.time(2));
}



//


// 计算TD-UT相关的计算

fn dt_ext(y: f64, jsd: f64) -> f64 {
    let dy = (y - 1820.0) / 100.0;
    -20.0 + jsd * dy.powi(2)
}

fn dt_calc(y: f64) -> f64 {
    let dt_at_len = constants::DT_AT.len();
    let y0 = constants::DT_AT[dt_at_len - 2]; // 表中最后一年
    let t0 = constants::DT_AT[dt_at_len - 1]; // 表中最后一年的deltatT
    if y >= y0 {
        let jsd = 31.0; // sjd是y1年之后的加速度估计。瑞士星历表jsd=31,NASA网站jsd=32,skmap的jsd=29
        if y > y0 + 100.0 {
            return dt_ext(y, jsd);
        }
        let v = dt_ext(y, jsd); // 二次曲线外推
        let dv = dt_ext(y0, jsd) - t0; // ye年的二次外推与te的差
        return v - (dv * (y0 + 100.0 - y)) / 100.0;
    }
    let d = &constants::DT_AT;
    for i in (0..d.len()).step_by(5) {
        if y < d[i + 5] {
            let t1 = ((y - d[i]) / (d[i + 5] - d[i])) * 10.0;
            let t2 = t1 * t1;
            let t3 = t2 * t1;
            return d[i + 1] + d[i + 2] * t1 + d[i + 3] * t2 + d[i + 4] * t3;
        }
    }
    0.0
}

pub fn dt_t(t: f64) -> f64 {
    dt_calc(t / 365.2425 + 2000.0) / 86400.0
}
