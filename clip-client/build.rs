fn main() {
    tonic_build::configure()
        .compile(
            &["../clip-ledger/proto/clip.proto"],
            &["../clip-ledger/proto/"],
        )
        .unwrap();
}
