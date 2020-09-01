#[cfg(target_os = "linux")]
mod linux {
    use std::fs::File;
    use std::convert::TryInto;
    use std::collections::BTreeSet;
    use std::io::{BufReader, BufRead};

    const INVALID_CORE_ID:     u32 = !0;
    const INVALID_PHYSICAL_ID: u32 = !0;

    #[derive(Copy, Clone)]
    pub struct Processor {
        id:          u32,
        core_id:     u32,
        physical_id: u32,
    }

    pub(super) fn processors(include_hyperthreads: bool) -> Vec<Processor> {
        let processor_count: usize = {
            const _SC_NPROCESSORS_ONLN: i32 = 84;

            extern "C" {
                fn sysconf(name: i32) -> i64;
            }

            unsafe {
                sysconf(_SC_NPROCESSORS_ONLN)
                    .try_into()
                    .expect("Invalid processor count returned by `sysconf`.")
            }
        };

        let mut processors: Vec<Processor> = Vec::with_capacity(processor_count);

        let cpuinfo = File::open("/proc/cpuinfo").expect("Failed to open `cpuinfo` file.");
        let cpuinfo = BufReader::new(cpuinfo);

        for line in cpuinfo.lines().filter_map(|line| line.ok()) {
            let mut splitted = line.split(':');

            let (key, value) = match (splitted.next(), splitted.next()) {
                (Some(key), Some(value)) => (key.trim(), value.trim()),
                _                        => continue,
            };

            let processor = processors.len().checked_sub(1);

            if key == "processor" {
                let id: u32 = value.parse().expect("Invalid processor ID.");

                if let Some(previous) = processor {
                    assert!(id > processors[previous].id, "Processor IDs are not increasing.");
                }

                processors.push(Processor {
                    id,
                    core_id:     INVALID_CORE_ID,
                    physical_id: INVALID_PHYSICAL_ID,
                });

                continue;
            }

            let processor = &mut processors[processor.expect("No valid processor.")];

            match key {
                "core id" => {
                    assert!(processor.core_id == INVALID_CORE_ID,
                            "Multiple core ID entries for processor {}.", processor.id);

                    processor.core_id = value.parse().expect("Invalid core ID.");
                }
                "physical id" => {
                    assert!(processor.physical_id == INVALID_PHYSICAL_ID,
                            "Multiple physcial ID entries for processor {}.", processor.id);

                    processor.physical_id = value.parse().expect("Invalid physical ID.");
                }
                _ => (),
            }
        }

        assert!(!processors.is_empty(), "No valid processors found on the system.");
        assert!(processors.len() == processor_count,
                "Number of detected processors differs from the `sysconf` returned value.");

        for processor in &processors {
            assert!(processor.core_id != INVALID_CORE_ID,
                    "Processor {} core ID not found.", processor.id);

            assert!(processor.physical_id != INVALID_PHYSICAL_ID,
                    "Processor {} physical ID not found.", processor.id);
        }

        if !include_hyperthreads {
            let mut physical_processors = BTreeSet::new();

            processors.retain(|processor| {
                physical_processors.insert((processor.core_id, processor.physical_id))
            });
        }

        processors
    }

    pub fn pin_to_processor(processor: &Processor) {
        const DEFAULT_SET_SIZE: usize = 1024;
        const USIZE_BITS:       usize = std::mem::size_of::<usize>() * 8;

        extern "C" {
            fn sched_setaffinity(pid: i32, cpuset_size: usize, cpuset: *const usize) -> i64;
        }

        let mut default_set_storage = [0usize; DEFAULT_SET_SIZE / USIZE_BITS];
        let mut special_set_storage;

        let processor_id: usize = processor.id.try_into().expect("Invalid processor ID.");

        let cpuset: &mut [usize] = if processor_id >= DEFAULT_SET_SIZE {
            let required_size = processor_id / USIZE_BITS + 1;
            let storage       = vec![0usize; required_size];

            special_set_storage = Some(storage);
            special_set_storage.as_mut().unwrap()
        } else {
            &mut default_set_storage
        };

        let idx = processor_id / USIZE_BITS;
        let bit = processor_id % USIZE_BITS;

        cpuset[idx] = 1 << bit;

        unsafe {
            assert!(sched_setaffinity(0, std::mem::size_of_val(cpuset), cpuset.as_ptr()) == 0,
                    "Pinning thread to specified processor failed.");
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    #[derive(Copy, Clone)]
    pub struct Processor {
        id:    u32,
        group: u16,
    }

    pub(super) fn processors(include_hyperthreads: bool) -> Vec<Processor> {
        unimplemented!()
    }

    pub fn pin_to_processor(processor: &Processor) {
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

        assert!(processor.id < 64, "Invalid processor ID.");

        let group_affinity = GROUP_AFFINITY {
            mask:     1 << processor.id,
            group:    processor.group,
            reserved: [0; 3],
        };

        unsafe {
            let result = SetThreadGroupAffinity(GetCurrentThread(), &group_affinity,
                                                std::ptr::null_mut());
            assert!(result == 1, "Pinning thread to specified processor failed.");
        }
    }
}

#[cfg(target_os = "linux")]
use linux as os;

#[cfg(target_os = "windows")]
use windows as os;

pub use os::{Processor, pin_to_processor};

pub fn logical() -> Vec<Processor> {
    os::processors(true)
}

pub fn physical() -> Vec<Processor> {
    os::processors(false)
}
