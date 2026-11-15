fn main() {
    println!("cargo:rerun-if-changed=assets/liminalnet.rc");
    println!("cargo:rerun-if-changed=assets/liminalnet.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/liminalnet.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("compile LiminalNet icon and version resource");
    }
    println!("cargo:rerun-if-env-changed=PCAP_LIBDIR");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && let Ok(path) = std::env::var("PCAP_LIBDIR")
    {
        println!("cargo:rustc-link-search=native={path}");
    }
}
