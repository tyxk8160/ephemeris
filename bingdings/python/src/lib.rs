//! Python bindings for ephemeris
#![warn(
    clippy::pedantic,
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::unwrap_used,
    clippy::map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility,
    rust_2021_compatibility
)]
#[allow(clippy::module_name_repetitions)]
use ::ephemeris as reph;
use ::ephemeris::JulianDate as RJulianDate;
use pyo3::{ prelude::*, wrap_pyfunction };

// create_exception, exceptions,types::PyList,

fn wrap(obj: &PyAny) -> Result<i32, PyErr> {
    let val = obj.call_method1("__and__", (0xffffffff_u32,))?;
    let val: u32 = val.extract()?;
    Ok(val as i32)
}

/// 儒略日转换
///
/// # Example
///
/// ```python
/// from ephemeris.ephemeris import *
/// j = JulianDate(2023,7,23.5)
/// print(j.jd())
///
/// x = JulianDate.jd2day(j)
/// print(x)
/// ```
#[derive(Debug)]
#[pyclass]
pub struct JulianDate(RJulianDate);

impl From<RJulianDate> for JulianDate {
    fn from(jd: RJulianDate) -> Self {
        Self(jd)
    }
}

#[pymethods]
impl JulianDate {
    #[new(text_signature = "(y,m,d)")]
    fn new(
        #[pyo3(from_py_with = "wrap")] y: i32,
        #[pyo3(from_py_with = "wrap")] m: i32,
        d: f64
    ) -> PyResult<Self> {
        let date = RJulianDate::from_day(y, m, d);

        Ok(Self(date))
    }

    #[pyo3(text_signature = "()")]
    fn jd(&self) -> f64 {
        self.0.jd
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0))
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let cls_name = slf.get_type().name()?;
        Ok(format!("{}({:?})", cls_name, slf.borrow().0))
    }

    /// 儒略日转公历
    ///
    /// # Argument
    /// - `jd`: 儒略日
    /// 返回值是公历（年，月，日）
    #[staticmethod]
    fn jd2day(jd: f64) -> (i32, i32, f64) {
        let (y, m, d) = RJulianDate::jd2day(jd);
        (y, m, d)
    }
}

/// 农历相关的结构
///
#[derive(Debug)]
#[pyclass]
pub struct YearCalender(reph::lunnar::YearCalender);

impl From<reph::lunnar::YearCalender> for YearCalender {
    fn from(yd: reph::lunnar::YearCalender) -> Self {
        Self(yd)
    }
}

#[pymethods]
impl YearCalender {
    #[new(text_signature = "(y)")]
    fn new(y: i32) -> PyResult<Self> {
        let a = reph::lunnar::YearCalender::new(y);
        Ok(Self(a))
    }

    pub fn zq(&self) -> PyResult<[f64; 25]> {
        Ok(self.0.zq)
    }

    pub fn hs(&self) -> PyResult<[f64; 15]> {
        Ok(self.0.hs)
    }

    pub fn year(&self) -> PyResult<i32> {
        Ok(self.0.year)
    }

    pub fn lunar_month(&self) -> PyResult<[i32; 15]> {
        Ok(self.0.lunar_month)
    }

    pub fn pe1(&self) -> PyResult<f64> {
        Ok(self.0.pe1)
    }

    pub fn pe2(&self) -> PyResult<f64> {
        Ok(self.0.pe2)
    }

    /// 相关函数
    ///
    /// 从日期构造年历数据
    #[staticmethod]
    pub fn from_date(year: i32, m: i32, d: f64) -> PyResult<Self> {
        let y = reph::lunnar::YearCalender::from_date(year, m, d);
        Ok(Self(y))
    }

    /// 获取农历的第n个月的信息
    ///
    /// # Argument
    /// - `n`: 月系数（从十一月起算）
    pub fn nth_month(&self, n: usize) -> PyResult<(i32, i32, i32, i32)> {
        let a = self.0.nth_month(n);
        Ok(a)
    }

    /// 获取第n个节气信息
    ///
    /// 获取从小雪开始第n个节气信息,返回儒略日天数
    pub fn nth_q24(&self, n: usize) -> PyResult<f64> {
        let a = self.0.nth_q24(n);
        Ok(a)
    }

    /// 获取年历信息
    ///
    /// 获取年历信息，主要输出十二月月名，24节气等信息
    pub fn info(&self) -> PyResult<()> {
        self.0.display();
        Ok(())
    }
}

#[derive(Debug)]
#[pyclass]
pub struct SolorDate(reph::lunnar::SolorDate);

impl From<reph::lunnar::SolorDate> for SolorDate {
    fn from(sd: reph::lunnar::SolorDate) -> Self {
        Self(sd)
    }
}

#[pymethods]
impl SolorDate {
    /// 构造日期
    #[new(text_signature = "(y, m,d)")]
    fn new(y: i32, m: i32, d: i32) -> PyResult<SolorDate> {
        let a = reph::lunnar::SolorDate(y, m, d);
        Ok(Self(a))
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let a = slf.borrow();
        let a = format!("SolorDate({},{},{})", a.0.0, a.0.1, a.0.2);
        Ok(a)
    }

    fn __str__(&self) -> PyResult<String> {
        let a: String = format!("{}-{}-{}", self.0.0, self.0.1, self.0.2);
        Ok(a)
    }

    /// 公历转农历
    fn to_lunar_date(&self) -> PyResult<LunarDate> {
        let ld = self.0.to_lunar_date();
        Ok(LunarDate::from(ld))
    }

    /// 计算前后后一个节气
    ///
    /// 计算公历 前一个节、或气， 或者后一个节或气, 0表示气，1表示节，返回节气的序号, 前一个d=0, 后一个d=1 返回值第一个为节气，
    ///  第一个为精确值 该方法在四柱中推算行运年份有用
    pub fn jq24(&self, jq_type: i32, d: usize) -> PyResult<(f64, usize)> {
        let jq = self.0.jq24(jq_type, d);
        Ok(jq)
    }

    /// 计算四柱
    ///
    /// 说明：是按照东八区算的
    pub fn sizhu(&self, t: f64) -> PyResult<(GanZhi, GanZhi, GanZhi, GanZhi)> {
        let (y, m, d, t) = self.0.sizhu(t);
        Ok((GanZhi::from(y), GanZhi::from(m), GanZhi::from(d), GanZhi::from(t)))
    }
}

// 干支
#[derive(Debug)]
#[pyclass]
pub struct GanZhi(reph::lunnar::GanZhi);

impl From<reph::lunnar::GanZhi> for GanZhi {
    fn from(gz: reph::lunnar::GanZhi) -> Self {
        Self(gz)
    }
}

#[pymethods]
impl GanZhi {
    #[new(text_signature = "(g,z)")]
    fn new(
        #[pyo3(from_py_with = "wrap")] g: i32,
        #[pyo3(from_py_with = "wrap")] z: i32
    ) -> PyResult<Self> {
        let a = reph::lunnar::GanZhi(g, z);
        Ok(Self(a))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.0))
    }

    fn gan(&self) -> PyResult<&str> {
        Ok(self.0.gan())
    }

    fn zhi(&self) -> PyResult<&str> {
        Ok(self.0.zhi())
    }

    fn inc(&self) -> PyResult<Self> {
        Ok(Self::from(self.0.inc()))
    }

    fn dec(&self) -> PyResult<Self> {
        Ok(Self::from(self.0.dec()))
    }
}

// MonthCalender
#[derive(Debug)]
#[pyclass]
pub struct MonthCalender(reph::lunnar::MonthCalender);

impl From<reph::lunnar::MonthCalender> for MonthCalender {
    fn from(r: reph::lunnar::MonthCalender) -> Self {
        Self(r)
    }
}

#[pymethods]
impl MonthCalender {
    #[new(text_signature = "(y,m)")]
    fn new(
        #[pyo3(from_py_with = "wrap")] y: i32,
        #[pyo3(from_py_with = "wrap")] m: i32
    ) -> PyResult<Self> {
        let r = reph::lunnar::MonthCalender::new(y, m);
        Ok(Self(r))
    }

    #[getter]
    fn years(&self) -> PyResult<i32> {
        let r = self.0.years;
        Ok(r)
    }

    #[getter]
    fn month(&self) -> PyResult<i32> {
        let r = self.0.month;
        Ok(r)
    }

    #[getter]
    fn firt_jd(&self) -> PyResult<f64> {
        let r = self.0.firt_jd;
        Ok(r)
    }

    #[getter]
    fn days(&self) -> PyResult<i32> {
        let r = self.0.days;
        Ok(r)
    }

    #[getter]
    fn lunar_date(&self) -> PyResult<LunarDate> {
        let r = self.0.lunnar_date;
        Ok(LunarDate::from(r))
    }
}

/// 农历年月日
#[derive(Debug)]
#[pyclass]
pub struct LunarDate(reph::lunnar::LunarDate);

impl From<reph::lunnar::LunarDate> for LunarDate {
    fn from(a: reph::lunnar::LunarDate) -> Self {
        Self(a)
    }
}

#[pymethods]
impl LunarDate {
    #[new(text_signature = "(y,m,d,leap)")]
    fn new(
        #[pyo3(from_py_with = "wrap")] y: i32,
        #[pyo3(from_py_with = "wrap")] m: i32,
        #[pyo3(from_py_with = "wrap")] d: i32,
        #[pyo3(from_py_with = "wrap")] leap: i32
    ) -> PyResult<Self> {
        let a = reph::lunnar::LunarDate(y, m, d, leap);
        Ok(Self(a))
    }

    fn to_solor_date(&self) -> PyResult<SolorDate> {
        let r = self.0.to_solor_date();
        Ok(SolorDate::from(r))
    }

    fn __str__(&self) -> PyResult<String> {
        let reph::lunnar::LunarDate(y, m, d, leap) = self.0;
        Ok(format!("{}年{}{}月{}日", y, if leap != 0 { "闰" } else { "" }, m, d))
    }
}

#[derive(Debug)]
#[pyclass]
pub struct DateDetail(reph::lunnar::DateDetail);

#[pymethods]
impl DateDetail {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.0))
    }

    #[getter]
    fn week(&self) -> PyResult<i32> {
        Ok(self.0.week)
    }

    #[getter]
    fn day(&self) -> PyResult<i32> {
        Ok(self.0.day)
    }

    #[getter]
    fn lunar(&self) -> PyResult<LunarDate> {
        Ok(LunarDate::from(self.0.lunar))
    }

    #[getter]
    fn date_gz(&self) -> PyResult<GanZhi> {
        Ok(GanZhi::from(self.0.date_gz))
    }

    #[getter]
    fn month_gz(&self) -> PyResult<GanZhi> {
        Ok(GanZhi::from(self.0.month_gz))
    }

    #[getter]
    fn jq24(&self) -> PyResult<i32> {
        Ok(self.0.jq24)
    }
}

/// 根据粗略的节气的儒略日返回精确的儒略日
#[pyfunction]
pub fn qi_accurate2(jd: f64) -> PyResult<f64> {
    Ok(reph::lunnar::qi_accurate2(jd))
}

/// 根据朔日粗略的儒略日返回精确的儒略日
#[pyfunction]
pub fn so_accurate2(jd: f64) -> PyResult<f64> {
    Ok(reph::lunnar::so_accurate2(jd))
}

// 天文历以及星座计算

/// 星体
#[derive(Debug, Clone, Copy)]
#[pyclass]
pub enum CelestialBody {
    Earth,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    Sun,
    Moon,
}

impl From<CelestialBody> for reph::astronomy::CelestialBody {
    fn from(c: CelestialBody) -> Self {
        match c {
            CelestialBody::Earth => Self::Earth,
            CelestialBody::Mercury => Self::Mercury,
            CelestialBody::Venus => Self::Venus,
            CelestialBody::Mars => Self::Mars,
            CelestialBody::Jupiter => Self::Jupiter,
            CelestialBody::Saturn => Self::Saturn,
            CelestialBody::Uranus => Self::Uranus,
            CelestialBody::Neptune => Self::Neptune,
            CelestialBody::Pluto => Self::Pluto,
            CelestialBody::Sun => Self::Sun,
            CelestialBody::Moon => Self::Moon,
        }
    }
}

impl From<reph::astronomy::CelestialBody> for CelestialBody {
    fn from(c: reph::astronomy::CelestialBody) -> Self {
        match c {
            reph::astronomy::CelestialBody::Earth => Self::Earth,
            reph::astronomy::CelestialBody::Mercury => Self::Mercury,
            reph::astronomy::CelestialBody::Venus => Self::Venus,
            reph::astronomy::CelestialBody::Mars => Self::Mars,
            reph::astronomy::CelestialBody::Jupiter => Self::Jupiter,
            reph::astronomy::CelestialBody::Saturn => Self::Saturn,
            reph::astronomy::CelestialBody::Uranus => Self::Uranus,
            reph::astronomy::CelestialBody::Neptune => Self::Neptune,
            reph::astronomy::CelestialBody::Pluto => Self::Pluto,
            reph::astronomy::CelestialBody::Sun => Self::Sun,
            reph::astronomy::CelestialBody::Moon => Self::Moon,
        }
    }
}

#[pymethods]
impl CelestialBody {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[derive(Debug)]
#[pyclass]
pub struct PlanetCoordinates(reph::astronomy::PlanetCoordinates);

impl From<reph::astronomy::PlanetCoordinates> for PlanetCoordinates {
    fn from(r: reph::astronomy::PlanetCoordinates) -> Self {
        Self(r)
    }
}

#[pymethods]
impl PlanetCoordinates {
    #[getter]
    pub fn eclon(&self) -> PyResult<f64> {
        Ok(self.0.eclon)
    }
    #[getter]
    pub fn eclat(&self) -> PyResult<f64> {
        Ok(self.0.eclat)
    }
    #[getter]
    pub fn a_lon(&self) -> PyResult<f64> {
        Ok(self.0.a_lon)
    }
    #[getter]
    pub fn a_lat(&self) -> PyResult<f64> {
        Ok(self.0.a_lat)
    }
    #[getter]
    pub fn a_ra(&self) -> PyResult<f64> {
        Ok(self.0.a_ra)
    }
    #[getter]
    pub fn a_dec(&self) -> PyResult<f64> {
        Ok(self.0.a_dec)
    }
    #[getter]
    pub fn r(&self) -> PyResult<f64> {
        Ok(self.0.r)
    }
    #[getter]
    pub fn d_e(&self) -> PyResult<f64> {
        Ok(self.0.d_e)
    }
    #[getter]
    pub fn lt(&self) -> PyResult<f64> {
        Ok(self.0.lt)
    }
    #[getter]
    pub fn st_ra(&self) -> PyResult<f64> {
        Ok(self.0.st_ra)
    }
    #[getter]
    pub fn st_dec(&self) -> PyResult<f64> {
        Ok(self.0.st_dec)
    }
    #[getter]
    pub fn dist(&self) -> PyResult<f64> {
        Ok(self.0.dist)
    }
    #[getter]
    pub fn az(&self) -> PyResult<f64> {
        Ok(self.0.az)
    }
    #[getter]
    pub fn alt(&self) -> PyResult<f64> {
        Ok(self.0.alt)
    }
    #[getter]
    pub fn sid_time(&self) -> PyResult<(f64, f64)> {
        Ok(self.0.sid_time)
    }
    #[getter]
    pub fn body(&self) -> PyResult<CelestialBody> {
        Ok(CelestialBody::from(self.0.body))
    }

    fn __str__(&self)->PyResult<String>{
        Ok(format!("{}", self.0))
    }
}


#[derive(Debug)]
#[pyclass]
pub struct Hourse(reph::astronomy::Hourse);
impl From<reph::astronomy::Hourse> for Hourse{
    fn from(r: reph::astronomy::Hourse) -> Self {
    
        Self(r)
    }
}

#[pymethods]
impl Hourse{
    #[new(text_signature="(jd,tz,lon,lat)")]
    pub fn new(jd: f64, tz: f64, lon: f64, lat: f64)->PyResult<Self>{
        Ok(Self(reph::astronomy::Hourse::new(jd, tz, lon, lat)))
    }

    #[getter]
    fn t(& mut self) -> PyResult<f64>{
        Ok(self.0.t())
    }
    
    #[getter]
    pub fn ra(& mut self) -> PyResult<f64>{
        Ok(self.0.ra())
    }
     
    #[getter]
    pub fn mc(& mut self) -> PyResult<f64>{
        Ok(self.0.mc())
    }

    #[getter] 
    pub fn asc(& mut self) -> PyResult<f64>{
        Ok(self.0.asc())
    }

    #[getter]
    pub fn ep(& mut self) -> PyResult<f64>{
        Ok(self.0.ep())
    }
}


/// 计算天体坐标
/// 
/// 
#[pyfunction]
pub fn calculate_celestial_body(
    body: CelestialBody,
    jd: f64,
    tz: f64,
    lon: f64,
    lat: f64
) -> PyResult<PlanetCoordinates>{
    let body = reph::astronomy::CelestialBody::from(body);
    let r = reph::astronomy::calculate_celestial_body(body,jd,tz,lon,lat);
    Ok(PlanetCoordinates::from(r))
}


/// 计算黄赤角
/// 
#[pyfunction]
pub fn obliquity(jd: f64) -> PyResult<f64>{
    Ok(reph::astronomy::obliquity(jd))
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn ephemeris(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<JulianDate>()?;

    // 日历模块
    let lunnar = PyModule::new(_py, "lunar")?;
    lunnar.add_class::<YearCalender>()?;
    lunnar.add_class::<MonthCalender>()?;
    lunnar.add_class::<SolorDate>()?;
    lunnar.add_class::<LunarDate>()?;
    lunnar.add_class::<DateDetail>()?;
    lunnar.add_class::<GanZhi>()?;
    lunnar.add_function(wrap_pyfunction!(qi_accurate2, m)?)?;
    lunnar.add_function(wrap_pyfunction!(so_accurate2, m)?)?;


    // 天文模块
    let astronomy = PyModule::new(_py, "astronomy")?;
    astronomy.add_class::<CelestialBody>()?;
    astronomy.add_class::<PlanetCoordinates>()?;
    astronomy.add_class::<Hourse>()?;
    astronomy.add_function(wrap_pyfunction!(obliquity, m)?)?;
    astronomy.add_function(wrap_pyfunction!(calculate_celestial_body,m)?)?;

    m.add_submodule(lunnar)?;
    m.add_submodule(astronomy)?;
    Ok(())
}

#[test]
fn test_aa() {
    // 创建一个新的 HashMap
    //    let mut map: HashMap<_, _> = HashMap::new();

    //    // 插入键值对
    //    map.insert("apple", Value::Integer(3));
    //    map.insert("banana", 2_i32);
    //    map.insert("orange", 5_f64);

    //    // 获取值
    //    let apple_count = map.get("apple");
    //    println!("The count of apples is {:?}", apple_count);

    //    // 遍历键值对
    //    for (key, value) in &map {
    //        println!("{}: {}", key, value);
    //    }
}
