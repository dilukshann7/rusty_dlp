use crate::model::EngineError;
use std::{path::Path, process::Stdio};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
};
use tokio_util::sync::CancellationToken;

/// Owns the entire process tree, including children started by FFmpeg.
pub struct ProcessTree {
    pub child: Child,
    #[cfg(unix)]
    group: i32,
    #[cfg(windows)]
    job: isize,
}

impl ProcessTree {
    pub fn spawn(executable: &Path, args: &[String]) -> Result<Self, EngineError> {
        let mut command = Command::new(executable);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        #[cfg(windows)]
        {
            // Assign the suspended process to a kill-on-close job BEFORE it can spawn descendants.
            use windows_sys::Win32::System::Threading::{CREATE_NO_WINDOW, CREATE_SUSPENDED};
            command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
        }
        let child = command.spawn()?;
        #[cfg(windows)]
        let mut child = child;
        let id = child
            .id()
            .ok_or_else(|| EngineError::Message("Could not identify the child process.".into()))?;
        #[cfg(windows)]
        {
            match windows_job::attach_and_resume(id) {
                Ok(job) => Ok(Self { child, job }),
                Err(error) => {
                    let _ = child.start_kill();
                    Err(error.into())
                }
            }
        }
        #[cfg(unix)]
        {
            Ok(Self {
                child,
                group: id as i32,
            })
        }
    }

    pub fn terminate(&mut self) {
        #[cfg(unix)]
        unsafe {
            // SAFETY: group was created for this child, never the application's process group.
            libc::kill(-self.group, libc::SIGKILL);
        }
        #[cfg(windows)]
        unsafe {
            // SAFETY: the job handle remains owned by this guard until Drop.
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job as _, 1);
        }
        let _ = self.child.start_kill();
    }
}
impl Drop for ProcessTree {
    fn drop(&mut self) {
        self.terminate();
        #[cfg(windows)]
        unsafe {
            // SAFETY: close our one owned handle once, after terminating the job.
            windows_sys::Win32::Foundation::CloseHandle(self.job as _);
        }
    }
}

#[cfg(windows)]
mod windows_job {
    use std::{
        io,
        mem::{size_of, zeroed},
        ptr::null,
    };
    use windows_sys::Win32::{
        Foundation::*,
        System::{Diagnostics::ToolHelp::*, JobObjects::*, Threading::*},
    };

    pub fn attach_and_resume(pid: u32) -> io::Result<isize> {
        // SAFETY: initialize documented Win32 structs and close every temporary owned handle.
        unsafe {
            let job = CreateJobObjectW(null(), null());
            if job.is_null() {
                return Err(io::Error::last_os_error());
            }
            let outcome = (|| {
                let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();
                limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &limits as *const _ as _,
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                ) == 0
                {
                    return Err(io::Error::last_os_error());
                }
                let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
                if process.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let assigned = AssignProcessToJobObject(job, process);
                let assign_error = io::Error::last_os_error();
                CloseHandle(process);
                if assigned == 0 {
                    return Err(assign_error);
                }
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if snapshot == INVALID_HANDLE_VALUE {
                    return Err(io::Error::last_os_error());
                }
                let mut entry: THREADENTRY32 = zeroed();
                entry.dwSize = size_of::<THREADENTRY32>() as u32;
                let mut found = false;
                let mut has_entry = Thread32First(snapshot, &mut entry) != 0;
                while has_entry {
                    if entry.th32OwnerProcessID == pid {
                        let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                        if !thread.is_null() {
                            found = ResumeThread(thread) != u32::MAX;
                            CloseHandle(thread);
                        }
                        break;
                    }
                    has_entry = Thread32Next(snapshot, &mut entry) != 0;
                }
                CloseHandle(snapshot);
                if !found {
                    return Err(io::Error::other(
                        "Could not resume the managed child process.",
                    ));
                }
                Ok(())
            })();
            if let Err(error) = outcome {
                TerminateJobObject(job, 1);
                CloseHandle(job);
                return Err(error);
            }
            Ok(job as isize)
        }
    }
}

/// Reads both pipes concurrently. Each line and the whole metadata response have hard bounds.
pub async fn capture(
    executable: &Path,
    args: &[String],
    cancellation: CancellationToken,
    timeout: std::time::Duration,
) -> Result<Vec<u8>, EngineError> {
    if cancellation.is_cancelled() {
        return Err(EngineError::Canceled);
    }
    let mut process = ProcessTree::spawn(executable, args)?;
    let stdout = process.child.stdout.take().unwrap();
    let stderr = process.child.stderr.take().unwrap();
    let output = async {
        let read_stdout = read_capped(stdout, 24 * 1024 * 1024);
        let read_stderr = read_capped(stderr, 256 * 1024);
        let (out, err, status) = tokio::join!(read_stdout, read_stderr, process.child.wait());
        let status = status?;
        if !status.success() {
            let err = err?;
            return Err(EngineError::Message(
                crate::engine::input::redact(&String::from_utf8_lossy(&err))
                    .chars()
                    .take(2000)
                    .collect(),
            ));
        }
        Ok(out?)
    };
    tokio::select! {
        _ = cancellation.cancelled() => Err(EngineError::Canceled),
        result = tokio::time::timeout(timeout, output) => result.map_err(|_| EngineError::Message("The tool timed out. Check connectivity and dependency versions.".into()))?,
    }
}

async fn read_capped(mut source: impl AsyncRead + Unpin, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let count = source.read(&mut chunk).await?;
        if count == 0 {
            break;
        }
        if bytes.len() + count > limit {
            return Err(std::io::Error::other(
                "Tool output exceeded the permitted size.",
            ));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok(bytes)
}

/// Bounded line framing that handles arbitrary pipe fragmentation without an unbounded read_line.
pub async fn drain_records(
    mut source: impl AsyncRead + Unpin,
    mut on_line: impl FnMut(&str),
) -> std::io::Result<()> {
    let mut pending = Vec::new();
    let mut chunk = [0; 8192];
    let mut dropping = false;
    loop {
        let count = source.read(&mut chunk).await?;
        if count == 0 {
            if !pending.is_empty() && !dropping {
                on_line(&String::from_utf8_lossy(&pending));
            }
            break;
        }
        for byte in &chunk[..count] {
            if *byte == b'\n' || *byte == b'\r' {
                if !pending.is_empty() && !dropping {
                    on_line(&String::from_utf8_lossy(&pending));
                }
                pending.clear();
                dropping = false;
            } else if !dropping {
                if pending.len() < 64 * 1024 {
                    pending.push(*byte);
                } else {
                    pending.clear();
                    dropping = true;
                }
            }
        }
    }
    Ok(())
}
