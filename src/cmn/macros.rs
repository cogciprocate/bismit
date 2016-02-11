// macro_rules! say_hello {
//     // `()` indicates that the macro takes no argument
//     () => (
//         // the macro will expand into the contents of this block
//         println!("Hello!");
//     )
// }

macro_rules! printlny {
    // ($fmt:expr) => ( print!(concat!("\x1b[93m", $fmt, "\x1b[0m", "\n")) );
    // ($fmt:expr, $($arg:tt)*) => ( print!(concat!("\x1b[93m", $fmt, "\x1b[0m", "\n"), $($arg)*) );
    ($fmt:expr) => ( print!(concat!(yellowify!($fmt), "\n")) );
    ($fmt:expr, $($arg:tt)*) => ( print!(concat!(yellowify!($fmt), "\n"), $($arg)*) );
}


macro_rules! yellowify {
    ($s:expr) => (concat!("\x1b[93m", $s, "\x1b[0m"));
}

