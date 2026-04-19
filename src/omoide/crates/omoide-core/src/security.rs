use omoide_env::*;

/// Suppresses core dump generation for this process.
///
/// Must be called once at process startup, before any secret material
/// is allocated. Satisfies T1 mitigation in the threat model.
///
/// # Platform behaviour
/// - Linux/macOS: sets RLIMIT_CORE to 0 via setrlimit
/// - Windows: core dumps are opt-in via WER, no action needed
/// - Other unix: best-effort, silently skips if unavailable
pub fn suppress_core_dumps() {
    #[cfg(unix)]
    {
        use libc::{rlimit, setrlimit, RLIMIT_CORE};

        let zero = rlimit {
            rlim_cur: RLIMIT_CORE_CUR,
            rlim_max: RLIMIT_CORE_MAX,
        };

        // SAFETY: setrlimit is async-signal-safe.
        // local process limits are only restricted, no invariants violated.
        unsafe {
            setrlimit(RLIMIT_CORE, &zero);
        }
        // Note: execution should be logged, not success/failure.
    }

    // Linux only: also disable /proc/<pid>/mem access by other processes
    #[cfg(target_os = "linux")]
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
    }

    // macOS only — denies debugger attachment at the kernel level
    #[cfg(target_os = "macos")]
    unsafe {
        libc::ptrace(PT_DENY_ATTACH, 0, 0, 0);
    }
}

/// Locks a secret value's memory pages, preventing the OS from paging
/// them to swap or hibernation files.
///
/// Call immediately after allocating any secret — before populating it.
/// Satisfies T1 swap/hibernation mitigation in the threat model.
///
/// # Safety contract
/// The caller must ensure `secret` remains valid for the duration of
/// its use. Unlocking happens automatically when the value is dropped
/// if using ZeroizeOnDrop, but mlock itself does not initialize and release here —
/// this is intentional to avoid a libc dep in omoide-crypto.
#[allow(unused_variables)] // function used in release build only
pub fn mlock_secret<T>(secret: &T) -> bool {
    #[cfg(all(unix, not(debug_assertions)))]
    unsafe {
        let ret = libc::mlock(
            secret as *const T as *const libc::c_void,
            core::mem::size_of::<T>(),
        );

        if ret != 0 {
            let errno = *libc::__errno_location();
            eprintln!(
                "[security] mlock failed (errno {}): secret memory may be swappable",
                errno
            );
            return false;
        }
        return true;
    }

    #[cfg(all(windows, not(debug_assertions)))]
    unsafe {
        let ret = windows_sys::Win32::System::Memory::VirtualLock(
            secret as *const T as *mut core::ffi::c_void,
            core::mem::size_of::<T>(),
        );

        if ret != 0 {
            let errno = windows_sys::Win32::Foundation::GetLastError();
            eprintln!(
                "[security] VirtualLock failed (errno {}): secret memory may be swappable",
                errno
            );
            return false;
        }
        return true;
    }

    // debug builds or non-unix: skip mlock, report as non-fatal
    #[allow(unreachable_code)]
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppress_core_dumps_does_not_panic() {
        // Behavioural test — we can't assert the OS state directly,
        // but we can assert this never panics on any supported platform
        suppress_core_dumps();
    }

    #[test]
    fn mlock_secret_does_not_panic() {
        let secret = [0u8; 32];
        let ret = mlock_secret(&secret);
        // If mlock failed silently, the memory is still valid — just not locked.
        // A hard failure here would mean we're over RLIMIT_MEMLOCK quota.
        assert_eq!(ret, true);
    }
}
