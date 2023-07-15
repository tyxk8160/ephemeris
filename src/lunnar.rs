/// 农历农历相关的函数
use crate::internal::lunnar::{ AstroyDate, calc_year_calendar, self };
use crate::internal::constants;

#[derive(Debug, Clone, Default)]
pub struct YearCalender {
    pub year: i32, // 年分
    pub zq: [f64; 25], // 中气计算, 返回儒历日, 0项是冬至
    pub hs: [f64; 15], // 合朔计算
    pub lunar_month: [i32; 15], // 记录月份名，leap| month
    pub lunar_leap: i32, // 闰月月序

    pub pe1: f64, // 补足小雪在11月场景
    pub pe2: f64,

    // private
    _days: [f64; 15],
    _ym: [String; 15],
}

impl YearCalender {
    pub fn new(year: i32) -> Self {
        let jd1 = AstroyDate::from_day(year, 1, 1.5).jd; // 计算真实的jd
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
        let jd = AstroyDate::from_day(year, m, d).jd;
        let mut y = YearCalender::new(year);
        // 判断是否要进入下一年年历
        if jd > y.zq[24] {
            y = YearCalender::new(year + 1);
        }

        y
    }

    /// 获取第n个月的信息（年，月，是否润月，天数）
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

    /// 获取第n个节气的序号，增加pe2, pe1, 补足农历11月缺口
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
                let (y, m, d) = AstroyDate::jd2day(self.hs[j]);
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
                let (y, m, d) = AstroyDate::jd2day(self.zq[i]);
                println!("节气：{} 日期:{}-{}-{} ", i, y, m, d as i32);
                i += 1;
            }
        }
    }
}

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
    pub fn new(year: i32, month: i32) -> Self {
        let firt_jd = AstroyDate::from_day(year, month, 1.5).jd;
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

        let d = AstroyDate::from_day(y1, m1, 1.5).jd - AstroyDate::from_day(y, m, 1.5).jd;
        d as i32
    }

    /// 获取日历信息,返回每一天信息
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

#[derive(Debug, Default, Clone)]
pub struct SolorDate(i32, i32, i32); // 年月日

impl SolorDate {
    /// 将公历转农历
    pub fn to_lunar_date(&self) -> LunarDate {
        let (lunar_date, _) = self.to_lunar_date_();
        lunar_date
    }

    pub fn to_lunar_date_(&self) -> (LunarDate, usize) {
        let jd = AstroyDate::from_day(self.0, self.1, (self.2 as f64) + 0.5).jd;
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

    /// 计算公历 前一个节、或气， 或者后一个节或气, 0表示气，1表示节，返回节气的序号,
    /// 前一个d=0, 后一个d=1 返回值第一个为节气， 第一个为精确值
    pub fn jq24(&self, jq_type: i32, d: usize) -> (f64, usize) {
        let jd = AstroyDate::from_day(self.0, self.1, (self.2 as f64) + 0.5).jd;
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
    pub fn sizhu(&self, t: f64) -> (GanZhi, GanZhi, GanZhi, GanZhi) {
        let jd = AstroyDate::from_day(self.0, self.1, (self.2 as f64) + t).jd;
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
pub fn qi_accurate2(jd: f64) -> f64 {
    let jd = jd - constants::J2000;
    let jd_ = lunnar::qi_accurate2(jd);

    jd_ + constants::J2000
}

/// 精确的朔月计算
pub fn so_accurate2(jd: f64) -> f64 {
    let jd: f64 = jd - constants::J2000;
    let jd_ = lunnar::so_accurate2(jd);
    jd_ + constants::J2000
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LunarDate(i32, i32, i32, i32); // 最后一个变量指定是否是闰月

impl LunarDate {
    /// 将农历转为公历
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
        let a = AstroyDate::jd2day(jd);
        SolorDate(a.0, a.1, a.2 as i32)
        
    }
}

/// 每日的日历信息
#[derive(Debug, Default, Clone)]
pub struct DateDetail {
    pub week: i32, // 周几
    pub day: i32, //号序数
    pub lunar: LunarDate, // 农历日期
    pub date_gz: GanZhi, // 日期干支
    pub month_gz: GanZhi, // 月份干支
    pub jq24: i32, // 节气
}

/// 干支
#[derive(Debug, Copy, Clone, Default)]
pub struct GanZhi(i32, i32); //  第一位干 ，第二位支， GanZhi(0,0) 表示甲子

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
        Self((self.0 + 1) % 10, (self.1 + 1) % 12)
    }

    pub fn dec(&self) -> Self {
        Self((self.0 - 1 + 9) % 10, (self.1 - 1 + 12) % 12)
    }

    pub fn gan(&self) -> &str {
        Self::GAN[(self.0 as usize) % 10]
    }

    pub fn zhi(&self) -> &str {
        Self::ZHI[(self.1 as usize) % 12]
    }
}

impl std::fmt::Display for GanZhi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.gan(), self.zhi())
    }
}

#[test]
fn test_calender() {
    let mut m = MonthCalender::new(2033, 12);
    let r = m.get_lunars();
    for i in r.iter() {
        println!("{:?} {}", i, i.date_gz);
    }
}

#[test]
fn test_solor_to_convert() {
    let x = SolorDate(2033, 12, 23).to_lunar_date();
    println!("{:?}", x);
}

#[test]
fn test_sizhu() {
    let d = SolorDate(2023, 11, 11);
    // 时间12点

    let sz = d.sizhu(0.5);

    println!("{} {} {} {}", sz.0, sz.1, sz.2, sz.3);
}

#[test]
fn test_convert_to_solor(){
    let a = LunarDate(2023,10,17,0);
    println!("{:?}", a.to_solor_date());

}
