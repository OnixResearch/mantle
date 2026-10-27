// SPDX-License-Identifier: AGPL-3.0-or-later
//! Keep a runnable guest task while synchronous filesystem work is in progress.
//! The pinned VMM treats an all-idle HLT as branch completion. This child has no
//! production authority and never uses SDK transport or changes workload facts.

pub struct GuestActivity {
    pid: libc::pid_t,
}

impl GuestActivity {
    pub fn start() -> Result<Self, String> {
        // The harness parent remains single-threaded: crash injection forks it
        // and calls the real allocating filesystem shell in those children.
        let parent = unsafe { libc::getpid() };
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err("guest activity child fork failed".into());
        }
        if pid == 0 {
            // Bound lifetime to this parent even if an external local runner dies.
            if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } != 0
                || unsafe { libc::getppid() } != parent
            {
                unsafe { libc::_exit(71) };
            }
            loop {
                // Stay runnable without starving the production parent/children.
                unsafe { libc::sched_yield() };
            }
        }
        Ok(Self { pid })
    }

    pub fn stop(mut self) -> Result<(), String> {
        let pid = self.pid;
        self.pid = 0;
        let killed = unsafe { libc::kill(pid, libc::SIGKILL) };
        let (waited, status) = reap(pid);
        if killed != 0 || waited != pid || !libc::WIFSIGNALED(status) || libc::WTERMSIG(status) != libc::SIGKILL {
            return Err("guest activity child was not observed killed and reaped".into());
        }
        Ok(())
    }
}

impl Drop for GuestActivity {
    fn drop(&mut self) {
        if self.pid > 0 {
            unsafe { libc::kill(self.pid, libc::SIGKILL) };
            let _ = reap(self.pid);
        }
    }
}

fn reap(pid: libc::pid_t) -> (libc::pid_t, libc::c_int) {
    loop {
        let mut status = 0;
        let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
        if waited < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        return (waited, status);
    }
}
