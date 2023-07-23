/// 农历农历相关的函数
use crate::internal::lunnar::{ JulianDate, calc_year_calendar, self };
use crate::internal::constants;

/// 年历相关结构
/// 
/// 年历结构，包含上一年的冬至到本年的冬至。比如`years`字段为2023年时，
/// 覆盖的范围包含2022年冬至-2023年冬至
/// 
/// # Example
/// ```
///use rust_ephemeris::lunnar::*;
/// let y = YearCalender::new(2023);
/// println!("中气:{:?}", y.zq); 
/// println!("合朔:{:?}", y.hs);
/// ```
/// 
/// 通过日期，获取改日期所属年历信息  
/// **注意**： 时间换算成了天数，比如12:00， 0.5天
/// ```
///use rust_ephemeris::lunnar::*;
/// let y =  YearCalender::from_date(2033, 12, 23.5);
/// println!("所属年历年份:{}", y.year); // 2024
/// ```
#[derive(Debug, Clone, Default)]
pub struct YearCalender {
    /// 年份
    pub year: i32, 
    /// 中气计算, 返回儒历日, 0项是冬至
    pub zq: [f64; 25], 
    /// 合朔计算,返回儒历日, 0项是十一月
    pub hs: [f64; 15], 
    /// 记录月份名，leap| month
    pub lunar_month: [i32; 15],
    /// 闰月月序 
    pub lunar_leap: i32, 
    /// 冬至前一个节气, pe1,pe2补足部分年份农历十一月初，
    /// 公历还是小雪刚过，直接严格照中气计算年历会导致计算闰月不方便 
    pub pe1: f64, 
    /// 冬至前两个节气
    pub pe2: f64,

    // private
    _days: [f64; 15],
    _ym: [String; 15],
}

impl YearCalender {
    pub fn new(year: i32) -> Self {
        let jd1 = JulianDate::from_day(year, 1, 1.5).jd; // 计算真实的jd
        let jd2 = jd1 - constants::J2000;

        let a = calc_year_calendar(jd2);

        let zq = a.zq.map(|x| x + constants::J2000);
        let hs = a.hs.map(|x| x + constants::J2000);

        Self {
            year: year,
            zq: zq,
            hs: hs,
            lunar_leap: a.leap,
            pe1: a.pe1 + constants::J2000,
            pe2: a.pe2 + constants::J2000,
            lunar_month: a.lunar_month,
            _days: a.dx,
            _ym: a.ym,
            ..Default::default()
        }
    }

    /// 从日期获取该日期所属的年历信息
    pub fn from_date(year: i32, m: i32, d: f64) -> Self {
        let jd = JulianDate::from_day(year, m, d).jd;
        let mut y = YearCalender::new(year);
        // 判断是否要进入下一年年历
        if jd > y.zq[24] {
            y = YearCalender::new(year + 1);
        }

        y
    }

    /// 获取第n个月(农历)的信息（年，月，是否润月，天数）
    /// 
    /// 第0个月固定为农历时十一月
    /// 
    /// # Example
    /// ```
    ///use rust_ephemeris::lunnar::*;
    /// let y = YearCalender::new(2023);
    /// const YM:[&str;12]=["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月","九月", "十月", "冬月", "腊月"];
    /// let (ly, lm, lleap,ldays) = y.nth_month(4);
    /// println!("年:{}, 月:{}{}, 天数:{}", ly, 
    /// if lleap!=0 {"闰"}else{""}, YM[(lm as usize + 11)%12 ], ldays);
    /// ```
    pub fn nth_month(&self, n: usize) -> (i32, i32, i32, i32) {
        let mut y = self.year;
        let t = self.lunar_month[n];

        let m = t & 0xff; // 月序号，移除置闰规则后， 第一个月是11月
        let leap = t >> 8;
        if m <= 2 {
            y -= 1; // 上一年十一月和十二月
        }
        let days = self._days[n] as i32;
        let m = match m {
            0 => 11,
            1 => 12,
            _ => m - 1,
        };
        (y, m, leap, days)
    }

    /// 获取第n个节气的序号
    /// 
    /// 为了计算简单，增加pe2, pe1, 补足农历11月缺口
    /// 
    /// # Example
    /// 参考精确气计算函数计算[`qi_accurate2`](crate::lunnar::qi_accurate2)
    pub fn nth_q24(&self, n: usize) -> f64 {
        match n {
            0 => self.pe2,
            1 => self.pe1,
            _ => self.zq[(n as usize) - 2],
        }
    }

    pub fn display(&self) {
        let js = 24;
        let ms = if self.lunar_leap > 0 { 13 } else { 12 };
        let mut i = 0;
        let mut j = 0;

        while i < js || j < ms {
            if self.hs[j] <= self.zq[i] {
                let (y, m, d) = JulianDate::jd2day(self.hs[j]);
                println!(
                    "农历月份：{} 天数:{} 日期:{}-{}-{}",
                    self._ym[j],
                    self._days[j],
                    y,
                    m,
                    d as i32
                );
                j += 1;
            } else {
                let (y, m, d) = JulianDate::jd2day(self.zq[i]);
                println!("节气：{} 日期:{}-{}-{} ", i, y, m, d as i32);
                i += 1;
            }
        }
    }
}

/// 月历结构
/// 
/// 主要功能获取月历 首日的儒略日，月的天数， 月首的农历信息等
/// 主要函数有计算某个月每一天的信息的函数 [get_lunars](crate::lunnar::MonthCalender::get_lunars)
#[derive(Debug, Default, Clone)]
pub struct MonthCalender {
    pub years: i32,
    pub month: i32,
    pub firt_jd: f64,
    pub days: i32, //月的天数
    pub lunnar_date: LunarDate,

    /// 年历信息
    _year: YearCalender,
    _mth: usize, // 首月月序数
}

impl MonthCalender {
    /// 通过年和月初始化一个月历函数
    pub fn new(year: i32, month: i32) -> Self {
        let firt_jd = JulianDate::from_day(year, month, 1.5).jd;
        let mut y = YearCalender::new(year);
        // 判断是否要进入下一年年历
        if firt_jd > y.zq[24] {
            y = YearCalender::new(year + 1);
        }

        let days = Self::days(year, month);

        let (lunar_date, yx) = SolorDate(year, month, 1).to_lunar_date_();
        Self {
            years: year,
            month: month,
            firt_jd: firt_jd,
            days: days,
            lunnar_date: lunar_date,
            _year: y,
            _mth: yx,
            ..Default::default()
        }
    }

    /// 获取月份的天数
    pub fn days(y: i32, m: i32) -> i32 {
        // 采用格林高历
        let mut m1 = m + 1;

        let y1 = if m1 > 12 {
            m1 = 1;
            y + 1
        } else {
            y
        };

        let d = JulianDate::from_day(y1, m1, 1.5).jd - JulianDate::from_day(y, m, 1.5).jd;
        d as i32
    }

    /// 获取日历信息,返回每一天信息
    /// 
    /// # Example
    /// ```
    ///use rust_ephemeris::lunnar::*;
    /// let mut m = MonthCalender::new(2033, 12);
    /// let r = m.get_lunars();
    ///  for i in r.iter() {
    ///        println!("{:?} {}", i, i.date_gz);
    ///  }
    /// ```
    pub fn get_lunars(&mut self) -> Vec<DateDetail> {
        let mut _i = 0_usize;
        let mut zq: f64 = self._year.pe2;
        while (zq as i32) <= (self.firt_jd as i32) {
            _i += 1;
            zq = self._year.nth_q24(_i);
        }

        let mut lunnar_date = self.lunnar_date; // 月第一天
        let mut _jq = _i - 1; // 记录节气序数
        let mut _jqn = _i; //
        let mut mth = self._mth + 1; //记录月序数
        let jqm = ((_jq as i32) + 1) / 2; // pe1 为子月，jqm =1
        let mut month_gz = GanZhi(
            ((self._year.year + 1) * 2 + jqm + 9) % 10,
            ((jqm + 11) % 12) as i32
        );
        let mut fjd: i32 = self.firt_jd as i32;

        let mut week = (fjd + 1) % 7;

        // 获取第一天gz
        let mut date_gz = GanZhi((fjd + 9) % 10, (fjd + 1) % 12);

        let mut result: Vec<DateDetail> = Vec::new();

        let jq24: i32 = if fjd == (self._year.nth_q24(_jq) as i32) { _jq as i32 } else { -1 };
        result.push(DateDetail {
            week: week,
            day: 1,
            lunar: lunnar_date,
            date_gz: date_gz,
            month_gz: month_gz,
            jq24: jq24,
        });
        for i in 1..self.days {
            fjd += 1;
            date_gz = date_gz.inc(); // 下一天的干支
            week = (week + 1) % 7; // 下一天的周历
            let mut jq24 = -1_i32;

            if (self._year.nth_q24(_jqn) as i32) == fjd {
                jq24 = _jqn as i32;
                if _jqn % 2 == 1 {
                    month_gz = month_gz.inc();
                    jq24 = _jqn as i32;
                }
                _jqn += 1;
                if _jqn == 27 {
                    // 冬至， 年历+1
                    self._year = YearCalender::new(self._year.year + 1);
                    _jqn = 1; // 月系数需要同步更改，因为冬至固定为子月，所以月序数固定为0
                    mth = 1;
                }
            }

            if (self._year.hs[mth] as i32) == fjd {
                let (m_year, m_moth, m_leap, _) = self._year.nth_month(mth);

                lunnar_date = LunarDate(m_year, m_moth, 1, m_leap);
                mth += 1;
            } else {
                lunnar_date = LunarDate(
                    lunnar_date.0,
                    lunnar_date.1,
                    lunnar_date.2 + 1,
                    lunnar_date.3
                );
            }

            let item = DateDetail {
                day: i + 1,
                date_gz: date_gz,
                week: week,
                month_gz: month_gz,
                lunar: lunnar_date,
                jq24: jq24,
                ..Default::default()
            };

            result.push(item);
        }

        result
    }
}

/// 公历
/// 
/// 主要提供公历转农历，以及八字计算等功能
/// # Example
/// `SolorDate(2023, 11,12)`表示公历2023-11-12
/// - 公历转农历
/// ```
///use rust_ephemeris::lunnar::*;
/// const YM:[&str;12]=["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月","九月", "十月", "冬月", "腊月"];
/// let x = SolorDate(2033, 12, 23).to_lunar_date();
/// println!("公历2023-12-23 农历：{}年 {}{} {}日",
/// x.0, if x.3!=0{ "闰"} else {""}, YM[(x.1 as usize + 11)%12], x.2 );
/// ```
/// - 公历转四柱八字
/// **注意**: 默认是采用东八区计算的，自己可以转真太阳时
/// ```
///use rust_ephemeris::lunnar::*;
/// let d = SolorDate(2023, 11, 11);
/// // 时间12点
/// let sz = d.sizhu(0.5);
/// println!("{} {} {} {}", sz.0, sz.1, sz.2, sz.3); // 癸卯 癸亥 癸酉 戊午
/// ```
#[derive(Debug, Default, Clone)]
pub struct SolorDate(pub i32, pub i32, pub i32); // 年月日

impl SolorDate {
    /// 将公历转农历
    /// 
    /// 用法参见[`SolorDate`](crate::lunnar::SolorDate)
    pub fn to_lunar_date(&self) -> LunarDate {
        let (lunar_date, _) = self.to_lunar_date_();
        lunar_date
    }

    pub fn to_lunar_date_(&self) -> (LunarDate, usize) {
        let jd = JulianDate::from_day(self.0, self.1, (self.2 as f64) + 0.5).jd;
        let y = YearCalender::from_date(self.0, self.1, (self.2 as f64) + 0.5);

        // 判断月份
        let mut yx = 0;
        while (jd as i32) > (y.hs[yx] as i32) {
            yx += 1;
        }
        let (ly, lm, lleap, _) = y.nth_month(yx - 1);
        let lunar_days = (jd - y.hs[yx - 1] + 1.0) as i32;
        (LunarDate(ly, lm, lunar_days, lleap), yx - 1)
    }

    /// 计算日期前一个或者后一个节或气
    /// 
    /// 计算公历 前一个节、或气， 或者后一个节或气, 0表示气，1表示节，返回节气的序号,
    /// 前一个d=0, 后一个d=1 返回值第一个为节气， 第一个为精确值
    /// 该方法在四柱中推算行运年份有用
    pub fn jq24(&self, jq_type: i32, d: usize) -> (f64, usize) {
        let jd = JulianDate::from_day(self.0, self.1, (self.2 as f64) + 0.5).jd;
        let y = YearCalender::from_date(self.0, self.1, (self.2 as f64) + 0.5);

        let mut _i = 0_usize;
        let mut zq: f64 = y.pe2;
        while (zq as i32) <= (jd as i32) {
            _i += 1;
            zq = y.nth_q24(_i);
        }
        // _i - 1 上一个节或气
        // _i 下一个节或气
        let mut prev_jq = _i - 1;
        if jq_type == 0 && prev_jq % 2 != 0 && jq_type == 1 && prev_jq % 2 != 1 {
            prev_jq -= 1;
        }
        let jq_ = prev_jq + 2 * d;
        let zq = y.nth_q24(jq_); // 粗略计算中气

        let jd_accurate: f64 = qi_accurate2(zq);
        (jd_accurate, jq_)
    }

    /// 计算年月日干支 采用浮点12 点为 0.5
    /// 
    /// 计算四柱，按照东八区时间推算，可以自行转为太阳时
    /// 用法参见[`SolorDate`](crate::lunnar::SolorDate)
    pub fn sizhu(&self, t: f64) -> (GanZhi, GanZhi, GanZhi, GanZhi) {
        let jd = JulianDate::from_day(self.0, self.1, (self.2 as f64) + t).jd;
        let y = YearCalender::from_date(self.0, self.1, (self.2 as f64) + 0.5);

        // 计算月份干支
        let (jq_acc, mut _jq) = self.jq24(1, 0);
        // 遇到恰好需要换节的情况需要进行修则正
        if jq_acc > jd {
            _jq -= 2;
        }
        let jqm = ((_jq as i32) + 1) / 2; // pe1 为子月，jqm =1
        let month_gz = GanZhi(((y.year + 1) * 2 + jqm + 9) % 10, ((jqm + 11) % 12) as i32);

        // 年份干支, 立春前算上一年
        let years = if _jq <= 5 { y.year - 1 } else { y.year };

        let year_gz = GanZhi((years + 6) % 10, (years + 8) % 12);

        // 算日期
        let jd_floor = jd.floor() as i32; // 本日或者前一日12点的jd
        let jd_r = jd - (jd_floor as f64);
        let mut date_gz = GanZhi((jd_floor + 9) % 10, (jd_floor + 1) % 12);
        if jd_r > 11.0 / 24.0 {
            // 23点属于第二天子时
            date_gz = date_gz.inc();
        }
        // 计算时支 time_gz, 23h 23min 属于子时

        let t_dz = (((t * 24.0 + 1.0) / 2.0).floor() as i32) % 12;
        let time_gz = GanZhi((date_gz.0 * 2 + t_dz) % 10, t_dz);
        (year_gz, month_gz, date_gz, time_gz)
    }
}

/// 精确计算中气
/// 
/// 先根据粗略的中气儒略日，计算精确的中气时刻
/// 
/// # Example
/// 计算2023年立春的时间点
/// ```
///use rust_ephemeris::lunnar::*;
///use rust_ephemeris::math_utils::Angle;
///use rust_ephemeris::JulianDate;
/// use std::f64::consts::PI;
/// let y = YearCalender::new(2023);
/// const SOLAR_TERMS: [&str; 24] = [
///    "小雪", "大雪", "冬至", "小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明",
/// "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋", "处暑", "白露",
/// "秋分", "寒露", "霜降", "立冬",
/// ];
/// let nth =5;
/// let jd = y.nth_q24(nth); // 立春计算
/// let jd = qi_accurate2(jd); // 精确时间计算
/// let (y,m,d) = JulianDate::jd2day(jd);
/// let d1 = d.floor() as i32;
/// let mut r=Angle::from_f64((d - d1 as f64)*2.0 * PI);
/// println!("{}时间为:{}-{}-{} {}", SOLAR_TERMS[nth], 
/// y, m, d1, r.time(2)); // 立春时间为:2023-2-4 10h 42m 31.35s
/// ```
/// 
pub fn qi_accurate2(jd: f64) -> f64 {
    let jd = jd - constants::J2000;
    let jd_ = lunnar::qi_accurate2(jd);

    jd_ + constants::J2000
}

/// 精确的朔月计算
/// 
/// 原理与精气计算类似，
/// 用法参考[`qi_accurate2`](crate::lunnar::qi_accurate2)
pub fn so_accurate2(jd: f64) -> f64 {
    let jd: f64 = jd - constants::J2000;
    let jd_ = lunnar::so_accurate2(jd);
    jd_ + constants::J2000
}

/// 农历
/// 
/// 0: 年， 1:月， 2:日，3:是否闰月，1为闰月，否则非闰月  
/// 主要功能是表示农历，以及实现将农历转为公历
/// 
/// # Example
/// 
/// ```
///use rust_ephemeris::lunnar::*;
/// let a = LunarDate(2023,10,17,0);
/// println!("{:?}", a.to_solor_date()); // SolorDate(2023, 11, 29)
/// ```
#[derive(Debug, Default, Copy, Clone)]
pub struct LunarDate(pub i32, pub i32, pub i32, pub i32); // 最后一个变量指定是否是闰月

impl LunarDate {
    /// 将农历转为公历
    /// 
    /// 用法参见[LunarDate](crate::lunnar::LunarDate)
    pub fn to_solor_date(&self)->SolorDate {
        let LunarDate(mut y, mut m, d, leap) = *self;

        if m >= 11 {
           
            y += 1;
        } 
        m  = (m+1)%12;
        m = leap<<8 | m;
     
        let yc = YearCalender::new(y);
        let index = yc.lunar_month.iter().position(|&x| x==m).unwrap();

        let mut jd = yc.hs[index];
        jd = jd  + (d - 1) as f64;
        let a = JulianDate::jd2day(jd);
        SolorDate(a.0, a.1, a.2 as i32)
        
    }
}

/// 每日的日历信息
/// 
/// 包含基本星期、日干支、月干支等信息
#[derive(Debug, Default, Clone)]
pub struct DateDetail {
    /// 星期， 0: 周日， 1: 周一， ..., 6:周六
    pub week: i32, // 周几
    /// 公历日期
    /// 比如11-23, `day=23`
    pub day: i32, //号序数
    /// 当天的农历日期
    pub lunar: LunarDate, // 农历日期
    /// 日干支
    pub date_gz: GanZhi, // 日期干支
    /// 月干支
    /// 
    /// **注意**: 习惯上玄学都是用节气做月的分界线，比如2月立春后才算**寅**月
    pub month_gz: GanZhi, // 月份干支
    /// 是否是节气， 非节气jq=-1，jq24表示节气的序数
    /// 
    /// 0: 小雪， 1: 大雪， ...
    pub jq24: i32, // 节气
}

/// 干支
/// 
/// GanZhi(0,0)表示甲子，(-1,1)表示单独一个支
/// # Example
/// 主要提供计算前一天干支。后一天干支的功能
/// ```
///use rust_ephemeris::lunnar::*;
/// let gz = GanZhi(0,0);// 甲子
/// println!("当前干支：{}", gz); // 当前前干支：甲子
/// let gz1 = gz.inc();  
/// println!("后一天干支:{}", gz1); // 后一天干支:乙丑
/// let gz2 = gz.dec();
/// println!("前一天干支:{}", gz2); // 前一天干支:癸亥

/// let g = GanZhi(7, -1);
/// println!("当前:{}, 前一天:{}， 后一天:{}", g, g.dec(), g.inc()); // 前一天干支:癸
/// ```
#[derive(Debug, Copy, Clone, Default)]
pub struct GanZhi(pub i32, pub i32); //  第一位干 ，第二位支， GanZhi(0,0) 表示甲子

impl GanZhi {
    const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    const ZHI: [&str; 12] = [
        "子",
        "丑",
        "寅",
        "卯",
        "辰",
        "巳",
        "午",
        "未",
        "申",
        "酉",
        "戌",
        "亥",
    ];

    pub fn inc(&self) -> Self {
        let g = match self.0 {
            -1 =>-1,
            _=>(self.0 + 1) % 10
        };
        let z = match self.1 {
            -1 =>-1,
            _=>(self.1 + 1) % 12  
        };
        Self(g, z)
    }

    pub fn dec(&self) -> Self {
        let g = match self.0 {
            -1 =>-1,
            _=>(self.0 + 9) % 10
        };
        let z = match self.1 {
            -1 =>-1,
            _=>(self.1 + 11) % 12  
        };
        Self(g, z)
    }

    pub fn gan(&self) -> &str {
        match self.0 {
            -1 => "",
            _ => Self::GAN[(self.0 as usize) % 10]
        }
       
    }

    pub fn zhi(&self) -> &str {
        match self.1 {
            -1=>"",
            _=> Self::ZHI[(self.1 as usize) % 12]
            
        }
    }
}

impl std::fmt::Display for GanZhi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.gan(), self.zhi())
    }
}
