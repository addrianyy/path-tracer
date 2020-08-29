use std::convert::TryInto;

#[cfg(target_os = "windows")]
pub fn pin_to_core(_core_id: usize) {
    // TODO
}

#[cfg(target_os = "linux")]
pub fn pin_to_core(core_id: usize) {
    const USIZE_BITS: usize = std::mem::size_of::<usize>() * 8;
    const GETTID:     usize = 186;

    let mut cpuset = [0usize; 1024 / USIZE_BITS];

    let idx = core_id / USIZE_BITS;
    let bit = core_id % USIZE_BITS;

    cpuset[idx] = 1 << bit;

    extern {
        fn sched_setaffinity(pid: usize, cpuset_size: usize, cpuset: *mut usize) -> i32;
        fn syscall(id: usize) -> isize;
    }

    unsafe {
        let tid = syscall(GETTID).try_into().expect("`gettid` returned invalid TID.");

        assert!(sched_setaffinity(tid, std::mem::size_of_val(&cpuset), cpuset.as_mut_ptr()) == 0,
                "Pinning thread to specified core failed.");
    }
}
