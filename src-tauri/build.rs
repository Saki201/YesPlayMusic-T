fn main() {
    // rust-embed 在编译期内嵌 ../dist。显式声明依赖，确保 dist 重新构建后
    // cargo 一定会重跑 build.rs（rust-embed 据此重新嵌入），避免增量编译时
    // 二进制里残留旧 dist 导致 release 白屏。
    println!("cargo:rerun-if-changed=../dist");
    tauri_build::build()
}
