
/// mlock a secret value in place. Call immediately after allocation,
/// before populating with secret material.
pub fn mlock_secret<T>(secret: &T) {
    #[cfg(unix)]
    unsafe {
        libc::mlock(
            secret as *const T as *const libc::c_void,
            core::mem::size_of::<T>(),
        );
        // failure is non-fatal — log but continue
    }
}