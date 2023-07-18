use std::f64::consts::PI;
use crate::internal::math_utils;
use crate::internal::constants;
use crate::internal::ephemeris::{compute_position, self};

/// 黄赤角计算
/// 
/// 基本黄赤道角计算，输入是ut时间的儒略日
/// 
/// # Argument
/// 
/// - `jd`: 儒略日，标准时间
pub fn obliquity(jd:f64)->f64{

    let t = (jd-constants::J2000)/36525.0;
    ephemeris::obliquity(t)
}


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
/// use ephemeris::JulianDate;
/// use std::f64::consts::PI;
/// let body = CelestialBody::Mercury;
/// let  jd = JulianDate::from_day(2023, 7,23.5).jd;
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
    let mut jd = jd - constants::J2000;

    jd = jd + tz / 24.0 + math_utils::dt_t(jd); // 转为标准力学时
    println!("xt:{} , jd:{}, l:{}, fa:{}", body as usize, jd, lon, lat);

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

/// 占星主要宫位计算
///
/// 在西洋占星中，主要有太阳、月亮以及九大行星等天体
/// (注：西洋占星用回归制，黄道十二宫只是一个时间概念，经度位置固定不变)
/// 此外还有一些重要概念，比如Asc(上升点)， Mc(中天)，借用术数概念将这些概念称为虚星
/// 上升点是指在天球上东方地平线与天球相交处的点，和术数中命宫概念相同
///
///
/// # Example
/// 
/// 计算 2023-3-21 18:30 东八区 121.45E， 31.216666666666665N  上升星座
/// ```
/// use std::f64::consts::PI;
/// use ephemeris::astronomy::*;
/// use ephemeris::{JulianDate, math_utils};
/// 
/// let jd = JulianDate::from_day(2023, 3,21.0+10.5/24.0).jd;
/// 
/// let lon = -121.45/180.0*PI;
/// let lat = 31.216666666666665/180.0 *PI;
/// let mut h = Hourse::new(jd, -8.0, lon, lat);
/// 
/// println!("T={}", h.t());
/// println!("RA={}", math_utils::Angle::from_f64(h.ra()).degress(2));
/// 
/// //上升点的计算
/// println!("ASC={}", math_utils::Angle::from_f64(h.asc()).degress(2));
/// // 东升点计算
/// println!("EP={}", math_utils::Angle::from_f64(h.ep()).degress(2));
/// 
/// // 中天点计算
/// println!("MC={}", math_utils::Angle::from_f64(h.mc()).degress(2));
/// 
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Hourse {
    pub jd: f64, // 儒略日， UT时间,
    pub tz: f64, // 时区
    pub lon: f64, // 经度
    pub lat: f64, // 纬度

    _ra: Option<f64>,
    _t: Option<f64>, // 儒略世纪， 相对于J1900
    _asc: Option<f64>, // 上升星座经度，弧度制
    _ep: Option<f64>, // 东升点计算
    _mc: Option<f64>, // 中天计算
}

impl Hourse {
    pub fn new(jd: f64, tz: f64, lon: f64, lat: f64) -> Self {
        Self { jd: jd, tz: tz, lon: lon, lat: lat, ..Default::default() }
    }
    
    /// 儒略世纪相对于1900年1月1日
    pub fn t(&mut self)->f64{
        match self._t {
            Some(a)=>a,
            None=>{      
            
                let _t = (self.jd - 2415020.0)/36525.0;

                self._t = Some(_t);
                _t

            }
        }
    }

    /// 赤经计算
    /// 
    /// 计算赤经，采用弧度制
    pub fn ra(&mut self)->f64{
        if let Some(ra_) = self._ra{
            return ra_;
        }
        let _t = self.t();

        let tmp_jd = self.jd  +0.5;
        let tim = (tmp_jd - tmp_jd.floor())*24.0; //获取时间（小时制）
        let lon = self.lon/PI * 180.0;

        let mut ra1= (6.6460656 + 2400.0513 * _t + 2.58E-5 * _t * _t + tim) * 15.0 - lon; // 角度制
        
        ra1 = math_utils::Angle::from_f64(ra1/180.0*PI).rad; // 弧度制下的ra
        self._ra = Some(ra1);
        ra1
    }

    /// 中天计算
    /// 
    pub fn mc(&mut self)->f64{
        if let Some(mc_) = self._mc{
            return mc_;
        }
        let ob = obliquity(self.jd);
        let  ra_ = self.ra();
        let m = ra_.tan();
        let n = ob.cos();
       
        let mut mc_ =m.atan2(n);
        if ra_>PI/2.0 && ra_ < 3.0*PI/2.0{
            mc_ += PI;
        }
        mc_ = math_utils::Angle::from_f64(mc_).rad;
        self._mc=Some(mc_);
        mc_

    }


    /// 上升点计算
    /// 
    /// 返回上升点经度，采用弧度制表示
    pub fn asc(&mut self)->f64{
        if let Some(asc_) = self._asc{
            return asc_;
        }
        let ob = obliquity(self.jd);
        let ra_ = self.ra();
        let lat = self.lat;
        let m = - ra_.sin() * ob.cos() - lat.tan() * ob.sin();
        let n = ra_.cos();

        let mut asc_ = n.atan2(m);
        asc_ = math_utils::Angle::from_f64(asc_).rad;
        self._asc = Some(asc_);
        asc_
    }


    /// 东升点计算
    pub fn ep(&mut self)->f64{
        if let Some(ep_) = self._ep{
            return ep_;
        }
        let ob = obliquity(self.jd);
        let ra_ = self.ra();
        let m =  - ra_.sin() * ob.cos();
        let n = ra_.cos();

        let mut ep_ = n.atan2(m);
        ep_ = math_utils::Angle::from_f64(ep_).rad;
        self._ep = Some(ep_);
        ep_
    }



}

