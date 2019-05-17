#![allow(unused)]
fn main() {
    let a = "gfgf-wefwf";
    let b = a.replace('-', " ");

    for c in b.chars() {
        println!("{}", c)
    }
}
