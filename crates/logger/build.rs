fn main() {
    let tokio_unstable = std::env::var_os("CARGO_CFG_TOKIO_UNSTABLE").is_some();

    if cfg!(feature = "profiling") && !tokio_unstable {
        println!(
            "cargo:warning=nx-logger `profiling` feature requires building with `--cfg tokio_unstable` \
             (set RUSTFLAGS=\"--cfg tokio_unstable\" or disable the feature)"
        );
    }

    println!("cargo:rerun-if-env-changed=CARGO_CFG_TOKIO_UNSTABLE");
}
