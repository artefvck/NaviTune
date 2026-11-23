pub mod advance_os;
pub mod gpu_telemetry;
pub mod macos_ioreg;

pub(crate) fn run_cmd_bounded(cmd: &mut std::process::Command, timeout_ms: u64) -> std::io::Result<std::process::Output> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let out_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut p) = stdout_pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });
    let err_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut p) = stderr_pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let status = loop {
        if let Some(st) = child.try_wait()? {
            break st;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "OS telemetry/platform command timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();
    Ok(std::process::Output { status, stdout, stderr })
}
