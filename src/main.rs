mod dice;
mod svg;

use dice::eval::{Compile, Eval};
use dice::parser::NamedList;
use std::env::args;

use crate::svg::render_progress_clock;

fn main() {
    let arguments = args();
    if arguments.len() > 1 {
        let segments: u8 = arguments
            .last()
            .expect("could not get argument.")
            .parse()
            .expect("could not parse number from argument.");
        println!("{}", segments);
        render_progress_clock(segments).expect("Could not render progress clock.");
    }
}
