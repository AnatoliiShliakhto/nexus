fn main() {
    let profiling_enabled = std::env::var_os("CARGO_FEATURE_PROFILING").is_some();
    let tokio_unstable = std::env::var_os("CARGO_CFG_TOKIO_UNSTABLE").is_some();

    if profiling_enabled && !tokio_unstable {
        println!(
            "cargo:warning={{ name | kebab_case }} `profiling` feature requires building with `--cfg tokio_unstable` \
             (set RUSTFLAGS=\"--cfg tokio_unstable\" or disable the feature)"
        );
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_PROFILING");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TOKIO_UNSTABLE");
}
