fn main() {
    #[cfg(feature = "proto")]
    {
        #[allow(clippy::expect_used)]
        prost_build::Config::new()
            .btree_map(["."])
            .compile_protos(&["proto/nulid.v1/nulid.proto"], &["proto/"])
            .expect("Failed to compile protobuf");
    }
}
