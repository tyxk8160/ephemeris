use crate::internal::math_utils;
use crate::internal::constants;
use crate::internal::ephemeris::compute_position;
/// 星体
///
/// 0=>地球， 1=>火星...10=>月球
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum CelestialBody {
    Earth, // 地球
    Mercury, // 水星
    Venus, // 金星
    Mars, // 火星
    Jupiter, // 木星
    Saturn, //土星
    Uranus, // 天王
    Neptune, //海王
    Pluto, // 冥王
    Sun, // 太阳
    #[default]
    Moon, // 月
}

impl From<usize> for CelestialBody {
    fn from(value: usize) -> Self {
        let resulut = match value {
            0 => Self::Earth,
            1 => Self::Mercury,
            2 => Self::Venus,
            3 => Self::Mars,
            4 => Self::Jupiter,
            5 => Self::Saturn,
            6 => Self::Uranus,
            7 => Self::Neptune,
            8 => Self::Pluto,
            9 => Self::Sun,
            10 => Self::Moon,
            _ => panic!("invalid value for CelestialBody "),
        };
        resulut
    }
}

/// 行星位置表示
///
/// 主要实现format函数，以及获取当前黄经信息等便捷函数
/// **注意**: 输出信息角度是360度制，和部分软件南纬用负数表示角度不一致
#[derive(Debug, Default)]
pub struct PlanetCoordinates {
    pub eclon: f64, // 黄经
    pub eclat: f64, // 黄纬

    pub a_lon: f64, // 视黄经
    pub a_lat: f64, // 视黄纬
    pub a_ra: f64, //视赤经
    pub a_dec: f64, // 视赤纬度

    pub r: f64, // 向径
    pub d_e: f64, // 地心距
    pub lt: f64, // 光行距
    pub st_ra: f64, // 站赤经
    pub st_dec: f64, // 站赤纬
    pub dist: f64, // 视距离
    pub az: f64, // 方位角
    pub alt: f64, // 高度角
    pub sid_time: (f64, f64), // 恒星时, 真、平
    pub body: CelestialBody, //星体
}

impl std::fmt::Display for PlanetCoordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        let body = self.body;
        if body != CelestialBody::Moon {
            s += &format!(
                "黄经一 {} 黄纬一 {} 向径一 {:.2}\n",
                math_utils::Angle::from_f64(self.eclon).degress(2), //rad2str(z[0], 0),
                math_utils::Angle::from_f64(self.eclat).degress(2), //rad2str(z[1], 0),
                self.r
            );
        }

        s += &format!(
            "视黄经 {} 视黄纬 {} 地心距 {:.2}\n",
            math_utils::Angle::from_f64(self.a_lon).degress(2), //rad2str(z[0], 0),
            math_utils::Angle::from_f64(self.a_lat).degress(2), //rad2str(z[1], 0),
            self.d_e
        );

        s += &format!(
            "视赤经 {} 视赤纬 {} 光行距 {:.2}\n",
            math_utils::Angle::from_f64(self.a_ra).time(2), //rad2str(z[0], 1),
            math_utils::Angle::from_f64(self.a_dec).degress(2), //rad2str(z[1], 0),
            self.lt
        );

        s += &format!(
            "站赤经 {} 站赤纬 {} 视距离 {:.2}\n",
            math_utils::Angle::from_f64(self.st_ra).time(2), //rad2str(z[0], 1),
            math_utils::Angle::from_f64(self.st_dec).degress(2), //rad2str(z[1], 0),
            self.dist
        );
        s += &format!(
            "方位角 {} 高度角 {}\n",
            math_utils::Angle::from_f64(self.az).degress(2), //rad2str(z[0], 0),
            math_utils::Angle::from_f64(self.alt).degress(2) // rad2str(z[1], 0)
        );
        let (gst_ping, gst) = self.sid_time;
        s += &format!(
            "恒星时 {}(平) {}(真)\n",
            math_utils::Angle::from_f64(gst_ping).time(2), //,rad2str(rad2mrad(gst_ping), 1),
            math_utils::Angle::from_f64(gst).time(2) //(rad2mrad(gst), 1)
        );
        write!(f, "{}", s)
    }
}

/// 计算天体位置信息
///
/// 主要包含黄经、黄纬度，视黄经、黄纬以及方位角信息
///
/// # Arguments
///
/// * `body` - 需要计算的天体
/// * `jd` - 儒略日
/// * `tz` - 时区， 比如东八区`tz=-8.0`
/// * `lon` - 经度信息， 注意是弧度制
/// * `lat` - 纬度信息，也是采用弧度制
/// 
/// # Example
/// -  计算2023-7-23 12:00水星星历
///  时区 东八区， 经度：116°23' 纬39°54'
/// 
/// ```
/// use ephemeris::astronomy::*;
/// use ephemeris::AstroyDate;
/// use std::f64::consts::PI;
/// let body = CelestialBody::Mercury;
/// let  jd = AstroyDate::from_day(2023, 7,23.5).jd;
/// let tz = -8.0; // 东八区
/// let lon = 116.0/180.0*PI + 23.0/60.0/180.0*PI;
/// // let lon = 1.9911297824990999;
/// let lat = 39.0/180.0*PI + 54.0/60.0/180.0*PI;
/// // let lat = 0.38746309394274114;
/// let pos = calculate_celestial_body(
///    body,
///    jd,
///    tz,
///    lon,
///    lat
/// );
/// println!("{}", pos);
/// ```
pub fn calculate_celestial_body(
    body: CelestialBody,
    jd: f64,
    tz: f64,
    lon: f64,
    lat: f64
) -> PlanetCoordinates {
    let mut jd = jd- constants::J2000;

    jd = jd + tz/24.0 + math_utils::dt_t(jd); // 转为标准力学时
    println!("xt:{} , jd:{}, l:{}, fa:{}",
     body as usize, jd, lon, lat);

    let (
        eclon_,
        eclat_,
        a_lon_,
        a_lat_,
        a_ra_,
        a_dec_,
        r_,
        d_e_,
        lt_,
        st_ra_,
        st_dec_,
        dist_,
        az_,
        alt_,
        sid_time_,
    ) = compute_position(body as usize, jd, lon, lat);

    let pos = PlanetCoordinates {
        body: body,
        eclon: eclon_,
        eclat: eclat_,
        a_lon: a_lon_,
        a_lat: a_lat_,
        a_ra: a_ra_,
        a_dec: a_dec_,
        r: r_,
        d_e: d_e_,
        lt: lt_,
        st_ra: st_ra_,
        st_dec: st_dec_,
        dist: dist_,
        az: az_,
        alt: alt_,
        sid_time: sid_time_,
    };
    pos
}
