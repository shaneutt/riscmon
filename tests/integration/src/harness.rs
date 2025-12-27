use std::{
    io::Write,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

// Command prompt timeout
const TIMEOUT: Duration = Duration::from_secs(10);

// Sleep time between shared buffer polls
const POLL_INTERVAL: Duration = Duration::from_millis(20);

// The sentinel that signals the REPL is ready for input.
const PROMPT: &str = "> ";

/// A handle to a running QEMU process with the riscmon kernel loaded.
pub struct QemuHarness {
    child: Child,
    stdin: std::process::ChildStdin,
    buf: Arc<Mutex<Vec<u8>>>,
}

impl QemuHarness {
    /// Spawn QEMU with the given kernel binary and wait for the first `"> "` prompt.
    pub fn spawn(kernel: &str) -> Self {
        let mut child = Command::new("qemu-system-riscv64")
            .args([
                "-machine",
                "virt",
                "-nographic",
                "-bios",
                "none",
                "-kernel",
                kernel,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn qemu-system-riscv64 — is it installed?");

        let stdin = child.stdin.take().expect("child stdin missing");
        let stdout = child.stdout.take().expect("child stdout missing");

        // Drain stdout in the background and accumulate it
        let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let buf_writer = Arc::clone(&buf);
        thread::spawn(move || {
            use std::io::Read;
            let mut stdout = stdout;
            let mut tmp = [0u8; 256];
            loop {
                match stdout.read(&mut tmp) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf_writer.lock().unwrap().extend_from_slice(&tmp[..n]);
                    }
                }
            }
        });

        let mut harness = Self {
            child: child,
            stdin: stdin,
            buf: buf,
        };

        // wait for boot
        harness.wait_for_prompt();

        harness
    }

    // Send a command to the REPL
    pub fn send(&mut self, cmd: &str) {
        write!(self.stdin, "{}\n", cmd).expect("failed to write to QEMU stdin");
        self.stdin.flush().expect("failed to flush QEMU stdin");
    }

    // Receive the response from a REPL command
    pub fn receive(&mut self) -> String {
        self.wait_for_prompt()
    }

    /// Kill the VM
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    // Poll until the prompt is ready
    fn wait_for_prompt(&mut self) -> String {
        let start = Instant::now();
        let mut seen: Vec<u8> = Vec::new();

        loop {
            if start.elapsed() >= TIMEOUT {
                let partial = String::from_utf8_lossy(&seen).into_owned();
                panic!(
                    "timed out waiting for QEMU prompt after {:?}; received so far:\n{}",
                    TIMEOUT, partial
                );
            }

            // Drain the accumulated output
            {
                let mut guard = self.buf.lock().unwrap();
                seen.extend_from_slice(&guard);
                guard.clear();
            }

            // Check for prompt sentinel
            if let Some(pos) = find_prompt(&seen) {
                let text = String::from_utf8_lossy(&seen[..pos]).into_owned();
                let remainder = seen[pos + PROMPT.len()..].to_vec();
                *self.buf.lock().unwrap() = remainder;
                return text;
            }

            thread::sleep(POLL_INTERVAL);
        }
    }
}

impl Drop for QemuHarness {
    fn drop(&mut self) {
        self.kill();
    }
}

// Locate the offset of the prompt sentinel
fn find_prompt(buf: &[u8]) -> Option<usize> {
    let prompt = PROMPT.as_bytes();
    buf.windows(prompt.len()).position(|w| w == prompt)
}
