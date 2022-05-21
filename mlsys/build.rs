use std::env;

fn build_cache() {
    cc::Build::new()
        .file("cxx/cache.cxx")
        .static_flag(true)
        .compile("mlcache");
}

fn build_android() {
    cc::Build::new()
        .cpp(true)
        .file("cxx/android.cxx")
        .cpp_link_stdlib("c++_static")
        .static_flag(true)
        .compile("mlandroid");
}

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "linux" || target_os == "android" {
        build_cache();
    }

    if target_os == "android" {
        build_android();
    }
}
