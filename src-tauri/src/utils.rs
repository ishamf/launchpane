use std::{future::Future, time::Instant};

use log::trace;


pub async fn trace_elapsed_time<T: Future>(label: &str, f: impl FnOnce() -> T) -> T::Output {
    let start = Instant::now();
    let result = f().await;
    let elapsed = start.elapsed();

    trace!("{} took {:.2?}", label, elapsed);
    
    result
}