#[cfg(not(target_arch = "wasm32"))]
use std::println;
/// MACROS
/// Cross-platform logging.
/// This is used to enable different debugging procedures depending on webbrowser or native development.
#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {{
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!($($t)*).into());
        #[cfg(not(target_arch = "wasm32"))]
        println!($($t)*);
    }};
}
