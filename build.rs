extern crate cc;

use std::path::Path;

fn main() {
    cc::Build::new()
        .file(Path::new("src").join("core").join("tsx.c"))
        .compile("libtsx.a");
}
