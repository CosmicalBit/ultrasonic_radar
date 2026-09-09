#[macro_export]
macro_rules! debug_code {
    ($($code:tt)*) => {
        #[cfg(debug_assertions)]
        $code
    };
}
