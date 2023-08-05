
use ::rust_ephemeris::lunnar as Rlunnar;
#[allow(clippy::module_name_repetitions)]
use wasm_bindgen::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct JsYearCalender {
    pub year: i32,
    pub zq: [f64; 25],
    pub hs: [f64; 15],
    pub lunar_month: [i32; 15],
    pub lunar_leap: i32,
    pub pe1: f64,
    pub pe2: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct MonthResponse{
    pub year: i32,
    pub month: i32,
    pub leap: i32,
    pub days:i32,
}



#[derive(Debug)]
#[wasm_bindgen]
pub struct YearCalender {
    inner: Rlunnar::YearCalender,
}

#[wasm_bindgen]
impl YearCalender {
    #[wasm_bindgen(constructor)]
    pub fn new(y: i32) -> Self {
        let r = Rlunnar::YearCalender::new(y);
        Self { inner: r }
    }

    #[wasm_bindgen]
    pub fn to_obj(&self) -> Result<JsValue, JsValue> {
        let r = JsYearCalender {
            year: self.inner.year,
            zq: self.inner.zq,
            hs: self.inner.hs,
            lunar_month: self.inner.lunar_month,
            lunar_leap: self.inner.lunar_leap,
            pe1: self.inner.pe1,
            pe2: self.inner.pe2,
        };

        Ok(serde_wasm_bindgen::to_value(&r)?)
    }

    #[wasm_bindgen]
    pub fn from_date(year: i32, m: i32, d: f64) -> Self{
        let r = Rlunnar::YearCalender::from_date(year, m, d);
        Self {inner:r}
    }

    #[wasm_bindgen]
    pub fn nth_month(&self, n:usize)->Result<JsValue, JsValue>{

        let (ly, lm, lleap,ldays) = self.inner.nth_month(n);
        let r = MonthResponse{
            year:ly,
            month:lm,
            leap:lleap,
            days:ldays
        };
        Ok(serde_wasm_bindgen::to_value(&r)?)
    }

    #[wasm_bindgen]
    pub fn nth_q24(&self, n: usize) -> f64{
        self.inner.nth_q24(n)
    }

    #[wasm_bindgen]
    pub fn display(&self){
        self.inner.display()
    }

}




#[derive(Debug,  Clone, Copy, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct JsLunarDate{
    pub year: i32,
    pub month: i32,
    pub day:i32,
    pub leap: i32,
}

impl From<Rlunnar::LunarDate> for JsLunarDate {
    fn from(value: Rlunnar::LunarDate) -> Self {
        Self { year: value.0, month: value.1, day: value.2, leap: value.3 }
    }
    
}


#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct JsGanzhi {
    pub gan: i32,
    pub zhi: i32,
}

impl From<Rlunnar::GanZhi> for JsGanzhi {
    fn from(value: Rlunnar::GanZhi) -> Self {
        Self { gan: value.0, zhi: value.1 }
    }
}


impl From<GanZhi> for JsGanzhi {
    fn from(value: GanZhi) -> Self {
        Self::from(value.inner)
    }
    
}


#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct JsDateDetail {
    pub week: i32,
    pub day: i32,
    pub lunar: JsLunarDate,
    pub date_gz: JsGanzhi,
    pub month_gz: JsGanzhi,
    pub jq24: i32,
}

impl From<Rlunnar::DateDetail> for JsDateDetail {

    fn from(value: Rlunnar::DateDetail) -> Self {
        Self { week: value.week, 
            day: value.day, lunar: JsLunarDate::from(value.lunar),
             date_gz: JsGanzhi::from(value.date_gz),
              month_gz: JsGanzhi::from(value.month_gz), 
              jq24: value.jq24 }
    }
    
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct JsMonthCalender {
    pub years: i32,
    pub month: i32,
    pub firt_jd: f64,
    pub days: i32,
    pub lunnar_date: JsLunarDate,
}


impl  From<Rlunnar::MonthCalender> for JsMonthCalender {
    fn from(value: Rlunnar::MonthCalender) -> Self {
        Self { years: value.years,
             month: value.month,
              firt_jd:value.firt_jd, 
              days: value.days,
               lunnar_date: JsLunarDate::from(value.lunnar_date) }
    }
    
}

#[wasm_bindgen]
pub struct MonthCalender{
    inner:Rlunnar::MonthCalender,
}


#[wasm_bindgen]
impl MonthCalender {
    #[wasm_bindgen(constructor)]
    pub fn new(year: i32, month: i32) -> Self{
        let  r = Rlunnar::MonthCalender::new(year, month);
        Self {inner: r}
    }


    #[wasm_bindgen]
    pub fn to_obj(&self) ->Result<JsValue, JsValue>{
        let r=JsMonthCalender::from(self.inner.clone());
        Ok(serde_wasm_bindgen::to_value(&r)?)
    }


    #[wasm_bindgen]
    pub fn  get_lunars(&mut self)->Result<JsValue, JsValue>{
        let data = self.inner.get_lunars();
        let r: Vec<JsDateDetail>= data.into_iter().map(JsDateDetail::from).collect();
        Ok(serde_wasm_bindgen::to_value(&r)?)
    }
    
}


#[derive(Clone)]
#[wasm_bindgen]
pub struct GanZhi{
    inner: Rlunnar::GanZhi,
}

impl  From<Rlunnar::GanZhi> for GanZhi {
    fn from(value: Rlunnar::GanZhi) -> Self {
        Self { inner: value }
    }
}


#[wasm_bindgen]
impl GanZhi {
    #[wasm_bindgen(constructor)]
    pub fn new(g:i32, z:i32)->Self{
      Self::from(Rlunnar::GanZhi(g,z))

    }
    
    #[wasm_bindgen]
    pub fn to_obj(&self)->Result<JsValue, JsValue>{

        let r = JsGanzhi::from(self.clone());
        Ok(serde_wasm_bindgen::to_value(&r)?)
        
    }

    #[wasm_bindgen]
    pub fn inc(&self)->Self{
        Self::from(self.inner.inc())
    }

    #[wasm_bindgen]
    pub fn dec(&self)->Self{
        Self::from(self.inner.dec())
    }

    #[wasm_bindgen]
    pub fn gan(&self)->String{
        self.inner.gan().to_string()
    }

    #[wasm_bindgen]
    pub fn zhi(&self) ->String{
        self.inner.zhi().to_string()
    }

    #[wasm_bindgen(js_name="toString")]
    pub fn display(&self)->String{
        format!("{}", self.inner)
    }

    
}