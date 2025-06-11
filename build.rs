// build.rs
fn main() {
    prost_build::compile_protos(&["protos/im.proto"], &["protos"]).unwrap();
}
