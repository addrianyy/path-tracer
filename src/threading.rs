use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone)]
pub struct PixelRange {
    pub start: usize,
    pub size:  usize,
}

pub struct PixelQueue {
    queue: Vec<PixelRange>,
    index: AtomicUsize,
}

impl PixelQueue {
    pub fn new(queue: Vec<PixelRange>) -> Self {
        Self {
            queue,
            index: AtomicUsize::new(0),
        }
    }

    pub fn pop(&self) -> Option<PixelRange> {
        let index = self.index.fetch_add(1, Ordering::SeqCst);

        if index < self.queue.len() {
            Some(self.queue[index])
        } else {
            None
        }
    }
}

pub fn core_count() -> usize {
    num_cpus::get()
}

#[cfg(target_os = "windows")]
pub fn pin_to_core(core_id: usize) {
    #[repr(C)]
    struct GROUP_AFFINITY {
        mask:     u64,
        group:    u16,
        reserved: [u16; 3],
    }

    extern {
        fn SetThreadGroupAffinity(thread: usize, group_affinity: *const GROUP_AFFINITY,
                                  previous_group_affinity: *mut GROUP_AFFINITY) -> i32;
        fn GetCurrentThread() -> usize;
    }

    let core_id = core_id as u64;

    let group_affinity = GROUP_AFFINITY {
        mask:     1 << (core_id % 64),
        group:    core_id as u16 / 64,
        reserved: [0; 3],
    };

    unsafe {
        let result = SetThreadGroupAffinity(GetCurrentThread(), &group_affinity,
                                            std::ptr::null_mut());
        assert!(result != 0, "Pinning thread to specified core failed.");
    }
}

#[cfg(target_os = "linux")]
pub fn pin_to_core(core_id: usize) {
    use std::convert::TryInto;

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
