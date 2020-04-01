fn main() {
    println!("cargo:rustc-link-search=native=lib"); // -L  lib dir
    println!("cargo:rustc-link-lib=static=nvme_release"); // -l  libnvme.a
    println!("cargo:rustc-link-lib=static=bidx"); // -l  libioeidx.a
    println!("cargo:rustc-link-lib=static=lightnvm"); // -l  liblightnvm.a
    println!("cargo:rustc-link-lib=static=jemalloc"); // -l  libjemalloc.a

    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=curl");
}
