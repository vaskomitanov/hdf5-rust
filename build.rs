#[cfg(feature = "lzf")]
fn build_lzf() {
    cc::Build::new()
        .warnings(false)
        .opt_level(3)
        .file("ext/lzf/lzf_c.c")
        .file("ext/lzf/lzf_d.c")
        .include("ext/lzf")
        .compile("lzf");
}

fn main() {
    hdf5_sys::emit_cfg_flags();
    #[cfg(feature = "lzf")]
    build_lzf();
}
