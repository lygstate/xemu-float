// build.rs

fn main() {
    cc::Build::new()
        .file("float_operators.c")
        .compile("float_operators");
}