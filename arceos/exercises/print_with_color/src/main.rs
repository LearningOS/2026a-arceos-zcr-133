#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // 起止转义序列，中间是绿色代码 32 和复位 0
    println!("\x1b[32m[WithColor]: Hello, Arceos!\x1b[0m");
}
