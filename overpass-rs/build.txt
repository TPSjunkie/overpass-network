// ./build.rs

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("bindings.rs");
    let mut file = File::create(&dest_path).unwrap();

    write!(
        file,
        r#"
        #![allow(non_snake_case)]
        #![allow(non_upper_case_globals)]
        #![allow(non_camel_case_types)]
        #![allow(unused_variables)]
        #![allow(dead_code)]
        #![allow(clippy::all)]
        #![allow(clippy::pedantic)]
        #![allow(clippy::nursery)]
        #![allow(clippy::cargo)]

        mod bindings;
        pub use bindings::*;

        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    "#
    )
    .unwrap();
}
