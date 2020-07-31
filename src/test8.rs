use unicode_width::UnicodeWidthStr;
use unicode_width::UnicodeWidthChar;

#[allow(dead_code)]
pub fn test8() {
    let d = std::fs::read("test.txt").unwrap();
    let s = String::from_utf8(d).unwrap();
    for l in s.split("\n") {
        println!("{}, {}", l.width(), l.width_cjk());
        let ch: Vec<i32> = l.chars().map(|c| match c.width() { Some(v) => v as i32, None => -1 } ).collect();
        println!("{:?}", ch);
    }


}