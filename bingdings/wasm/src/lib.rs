
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
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility,
    rust_2021_compatibility
)]
#[allow(clippy::module_name_repetitions)]
// use ::rust_ephemeris as reph;
use ::rust_ephemeris::JulianDate as RJulianDate;
use wasm_bindgen::prelude::*;


pub mod  lunnar;


#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct JsDay{
    pub year: i32,
    pub month: i32,
    pub day: f64
}


#[derive(Debug)]
#[wasm_bindgen]
pub struct JulianDate;

impl From<RJulianDate> for JulianDate {
    fn from(_: RJulianDate) -> Self {
        Self
    }
}

#[wasm_bindgen]
impl  JulianDate {
    #[wasm_bindgen(constructor)]
    pub fn new(jd:f64)->Self{
         Self::from(RJulianDate::new(jd))
    }

    /// wasm 不支持元组类型，故需要转为JsValue类型
    #[wasm_bindgen]
    pub fn jd2day(x: f64) -> Result<JsValue, JsValue> {
        let (x,y,z)=RJulianDate::jd2day(x);
        let r = JsDay{
            year:x,
            month:y,
            day:z
        };
        Ok(serde_wasm_bindgen::to_value(&r)?)

    }


    #[wasm_bindgen]
    pub fn from_day(y: i32, m: i32, d: f64) ->Self{

        Self::from(RJulianDate::from_day(y,m,d))
    }
    
}



#[derive(Debug)]
#[wasm_bindgen(getter_with_clone)]
pub struct Student {
    pub age: i32,
    pub name: String,
}

#[wasm_bindgen]
impl Student {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, age: i32) ->Self{
        Self {
            name: name.clone(),
            age: age,
        }
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, ephemeris!");
}
