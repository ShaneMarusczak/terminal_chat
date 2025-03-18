use crossterm::{ExecutableCommand, cursor};
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
            " \n \\o/ \n  |  \n / \\",
            " \n \\o  \n  |\\ \n / \\",
            " \n  o| \n /|  \n / \\",
            " \n  o/ \n /|  \n  \\",
            " \n  o  \n /|  \n  \\",
            " \n  o  \n |\\ \n  \\",
            " \n  o/ \n |\\ \n  /",
        ];
        let mut i = 0;
        while spinner_flag.load(Ordering::Relaxed) {
            print!(
                "\r\x1b[2K\x1b[36m{}\x1b[0m",
                spinner_states[i % spinner_states.len()]
            );
            stdout().execute(cursor::MoveUp(3)).unwrap();
            let _ = stdout().flush();
            i += 1;
            sleep(Duration::from_millis(150)).await;
        }

        for _ in 0..4 {
            print!("\r\x1b[2K"); // Clear current line
            stdout().execute(cursor::MoveDown(1)).unwrap();
        }
        stdout().execute(cursor::MoveUp(3)).unwrap();
    });
    (spinner_running, handle)
}
