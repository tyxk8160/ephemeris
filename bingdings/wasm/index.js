
import {greet, JulianDate, YearCalender, MonthCalender} from './pkg';



let a = JulianDate.jd2day(20481.3);

let r1 = new YearCalender(2013);
console.log(r1.to_obj());


let r2 = new MonthCalender(2023, 7);
console.log(r2.get_lunars())

console.log(`julian ${a}`)
debugger;
greet();
