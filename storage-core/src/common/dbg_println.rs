#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => {
        println!($($arg)*);
    }
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => {};
}
