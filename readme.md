# Rust ephemeris

Rust ephemeris是一个用rust编写的支持年份范围广，提供多语言绑定的天文历法库。适合广大的天文历法爱好者和术数爱好者。

## 特性

- 支持计算二十四节气的具体时间点

- 支持计算农历合朔日计算
- 支持公历/农历之间的转换
- 支持八大行星+冥王星+太阳+月亮位置计算，方便占星以及七政四余爱好者排盘
- 支持上升点、中天以及东升点等星宫计算
- 支持八字四柱计算
- 支持儒略日计算
- 支持儒略日计算
- 支持多语言绑定，目前提供了python绑定，计算支持wasm绑定以及C绑定



## 安装

克隆repo

```bash
git clone https://github.com/tyxk8160/ephemeris.git
```



## 使用

**rust用户**

在`Cargo.tmol`添加依赖

```tom
[dependencies]
ephemeris = { git = "https://github.com/tyxk8160/ephemeris.git", branch = "master" } # 从git拉取

```



- 公历转农历

  `SolorDate(2023, 11,12)`表示公历2023-11-12

  ```rust
  use ephemeris::lunnar::*;
  const YM:[&str;12]=["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月","九月", "十月", "冬月", "腊月"];
  let x = SolorDate(2033, 12, 23).to_lunar_date();
  println!("公历2023-12-23 农历：{}年 {}{} {}日",
  x.0, if x.3!=0{ "闰"} else {""}, YM[(x.1 as usize + 11)%12], x.2 );
  ```

  

- 公历转四柱
  **注意**: 默认是采用东八区计算的，自己可以转真太阳时

  ```rust
  use ephemeris::lunnar::*;
  let d = SolorDate(2023, 11, 11);
  // 时间12点
  let sz = d.sizhu(0.5);
  println!("{} {} {} {}", sz.0, sz.1, sz.2, sz.3); // 癸卯 癸亥 癸酉 戊午
  ```

- 农历转公历

  ```rust
  use ephemeris::lunnar::*;
  let a = LunarDate(2023,10,17,0);
  println!("{:?}", a.to_solor_date()); // SolorDate(2023, 11, 29)
  ```

- 行星位置计算

  计算2023-7-23 12:00水星星历 时区 东八区， 经度：116°23’ 纬39°54’

  ```rust
  use ephemeris::astronomy::*;
  use ephemeris::JulianDate;
  use std::f64::consts::PI;
  let body = CelestialBody::Mercury;
  let  jd = JulianDate::from_day(2023, 7,23.5).jd;
  let tz = -8.0; // 东八区
  let lon = 116.0/180.0*PI + 23.0/60.0/180.0*PI;
  // let lon = 1.9911297824990999;
  let lat = 39.0/180.0*PI + 54.0/60.0/180.0*PI;
  // let lat = 0.38746309394274114;
  let pos = calculate_celestial_body(
     body,
     jd,
     tz,
     lon,
     lat
  );
  
  println!("{}", pos);
  ```

- 星座计算

  计算 2023-3-21 18:30 东八区 121.45E， 31.216666666666665N 

  ```rust
  use std::f64::consts::PI;
  use ephemeris::astronomy::*;
  use ephemeris::{JulianDate, math_utils};
   
  let jd = JulianDate::from_day(2023, 3,21.0+10.5/24.0).jd;
   
  let lon = -121.45/180.0*PI;
  let lat = 31.216666666666665/180.0 *PI;
  let mut h = Hourse::new(jd, -8.0, lon, lat);
   
  println!("T={}", h.t());
  println!("RA={}", math_utils::Angle::from_f64(h.ra()).degress(2));
   
  //上升点的计算
  println!("ASC={}", math_utils::Angle::from_f64(h.asc()).degress(2));
  // 东升点计算
  println!("EP={}", math_utils::Angle::from_f64(h.ep()).degress(2));
   
  // 中天点计算
  println!("MC={}", math_utils::Angle::from_f64(h.mc()).degress(2));
  ```

更多使用方法参考文档

**python用户**

暂时项目没稳定，未添加到pypi中

- 编译

  ```bash
  cd <repo>/bindings/python
  python setup.py build
  ```

- 基础使用

  ```python
  import ephemeris.ephemeris as eph
  # 计算儒略日
  print(eph.JulianDate(2023,7,23))
  ```

  更多用户参考rust文档

## 文档

- 生成文档

```bash
cargo doc --no-deps --open
```



## FAQ

- **Q**:  为什么会对天文历感兴趣
- **A**：读《六壬大全》六十四课有一个天地二烦课，有一个月宿的起法口诀在不同的术数中存在部分的差异。而且根据字面意思，月宿是指的月亮的位置，且月宿周期为二十八天左右，与月亮公转周期接近。所以我考虑不用口诀起月宿，直接计算月亮位置，判断所属二十八星宿
- **Q**: 为什么会重复造轮子写一个天文历的库？

- **A**：大部分常见的历法库是1900年后，在看古书去校准时间多有不便。
- **Q**：为什么采用rust编写？
- **A**：天文历法是一个基础库，以后打算写一个排盘软件或者计算格局的东西。排盘适合自带丰富界面功能的js写，计算格局不需要界面自用采用python等脚本语言比较方便。rust有很好的三方库，很容易绑定到python 、wasm、c，是开发基础库很好的选择。

## 贡献

我们非常欢迎社区的贡献。如果您发现任何错误或想改进 Rust ephemeris，请提交一个 pull 请求或创建一个 issue。



## 致谢
Jean Meeus  - 因他的著作《天文算法》而为此项目提供了灵感。

Rust 社区 - 感谢他们在 Rust Astro 开发过程中提供的支持和帮助。

寿星天文历 - 大量的日历计算库支持的范围是1900年以后，寿星天文历是少量支持时间范围广，且精度有一定保证的适合东方人的历法

[openai](https://openai.com/) - 绝大部分代码是chatGTP从js代码或者一些数学公式翻译成rust, 节省了大量的搬砖时间

 