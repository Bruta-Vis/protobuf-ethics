fn main() {
    prost_build::compile_protos(&["proto/ethics.proto"], &["proto"]).unwrap();
}
