fn main() {
    tonic_build::compile_protos("proto/stream_custom.proto").unwrap();
}
