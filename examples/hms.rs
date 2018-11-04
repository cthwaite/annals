extern crate annals;
use annals::Scribe;

fn main() {
    let mut scribe = Scribe::default();
    scribe.load_cognates("texts/hms.yml").unwrap();
    for _i in 0..4 {
        println!("{}", scribe.gen("root").unwrap());
        println!("");
    }
}
