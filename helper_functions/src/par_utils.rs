#[macro_export]
macro_rules! par_iter {
    ($collection: expr) => {{
        #[cfg(target_os = "zkvm")]
        {
            $collection.iter()
        }

        #[cfg(not(target_os = "zkvm"))]
        {
            $collection.par_iter()
        }
    }};
}
