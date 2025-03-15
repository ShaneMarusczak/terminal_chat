use std::{
    io::{Write, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::time::{Duration, sleep};

pub async fn run_with_spinner<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let (spinner_running, spinner_handle) = start_spinner();
    let result = f.await;
    spinner_running.store(false, Ordering::Relaxed);
    let _ = spinner_handle.await;
    result
}

pub fn start_spinner() -> (Arc<AtomicBool>, tokio::task::JoinHandle<()>) {
    let spinner_running = Arc::new(AtomicBool::new(true));
    let spinner_flag = spinner_running.clone();
    let handle = tokio::spawn(async move {
        let spinner_states = [
            " └[-   ]┐   ",
            "  ┌[ -  ]┘  ",
            "   └[  - ]┐ ",
            "    ┌[   -]┘",
            "   └[  - ]┐ ",
            "  ┌[ -  ]┘  ",
            " └[-   ]┐   ",
        ];
        let mut i = 0;
        while spinner_flag.load(Ordering::Relaxed) {
            print!(
                "\r\x1b[36m{}\x1b[0m",
                spinner_states[i % spinner_states.len()]
            );
            let _ = stdout().flush();
            i += 1;
            sleep(Duration::from_millis(125)).await;
        }
    });
    (spinner_running, handle)
}
