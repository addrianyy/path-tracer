use std::collections::BTreeSet;

fn remove_hyperthreads<T, Y: Ord>(processors: &mut Vec<T>,
                                  mut get_core_id: impl FnMut(&T) -> Y) {
    let mut physical_processors = BTreeSet::new();

    processors.retain(|processor| {
        physical_processors.insert(get_core_id(processor))
    });
}

#[cfg(target_os = "linux")]
mod linux {
    use std::fs::File;
    use std::convert::TryInto;
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
            super::remove_hyperthreads(&mut processors, |p| p.core_id);
        }

        processors
    }

    pub fn pin_to_processor(processor: &Processor, force: bool) {
        // Linux scheduler is pretty good so unless caller forced us to pin to the processor
        // don't do it..
        if !force {
            return;
        }

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
    #[repr(C)]
    struct GROUP_AFFINITY {
        mask:     usize,
        group:    u16,
        reserved: [u16; 3],
    }

    #[derive(Copy, Clone)]
    pub struct Processor {
        id:      u32,
        group:   u16,
        core_id: u32,
    }

    pub(super) fn processors(include_hyperthreads: bool) -> Vec<Processor> {
        const RELATION_PROCESSOR_CORE: u32 = 0;
        const USIZE_SIZE:              usize = std::mem::size_of::<usize>();

        #[repr(C)]
        struct PROCESSOR_RELATIONSHIP {
            flags:            u8,
            efficiency_class: u8,
            reserved:         [u8; 20],
            group_count:      u16,
            //group_mask:     [GROUP_AFFINITY; 0],
        }

        #[repr(C)]
        struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX  {
            relationship: u32,
            size:         u32,
            processor:    PROCESSOR_RELATIONSHIP,
        }

        extern {
            fn GetLogicalProcessorInformationEx(relationship: u32, buffer: *mut u8,
                                                returned_length: *mut u32) -> i32;
        }

        let mut buffer: Vec<usize> = Vec::new();
        let mut length: u32;

        let mut tries = 0;

        loop {
            length = (buffer.len() * USIZE_SIZE) as u32;

            let result = unsafe {
                GetLogicalProcessorInformationEx(RELATION_PROCESSOR_CORE,
                                                 buffer.as_mut_ptr() as _, &mut length)
            };

            if result == 1 {
                break;
            }

            if tries > 5 {
                panic!("Getting logical processor information failed after {} tries.", tries);
            }

            buffer = vec![0usize; (length as usize + USIZE_SIZE - 1) / USIZE_SIZE];
            tries += 1;
        }

        let mut remaining = length as usize;
        let mut offset    = 0;
        let mut core_id   = 0;

        let mut processors = Vec::new();

        while remaining > 0 {
            assert!(remaining >= std::mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>(),
                    "Invalid amount of remaining bytes.");

            let information = unsafe {
                let ptr = buffer.as_ptr() as usize + offset;

                &*(ptr as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX)
            };

            let group_count = information.processor.group_count as usize;

            for index in 0..group_count {
                let group = unsafe {
                    let ptr = &information.processor as *const _ as usize +
                        std::mem::size_of::<PROCESSOR_RELATIONSHIP>() +
                        std::mem::size_of::<GROUP_AFFINITY>() * index;

                    &*(ptr as *const GROUP_AFFINITY)
                };

                let mask_size = std::mem::size_of::<usize>() as u32 * 8;

                for id in 0..mask_size {
                    if group.mask & (1 << id) != 0 {
                        processors.push(Processor {
                            id,
                            core_id,
                            group: group.group,
                        });
                    }
                }
            }

            let size = information.size as usize;

            offset    += size;
            remaining -= size;
            core_id   += 1;
        }

        if !include_hyperthreads {
            super::remove_hyperthreads(&mut processors, |p| p.core_id);
        }

        processors
    }

    pub fn pin_to_processor(processor: &Processor, _force: bool) {
        // Windows scheduler isn't very good so always pin to the specified processor.

        extern {
            fn SetThreadGroupAffinity(thread: usize, group_affinity: *const GROUP_AFFINITY,
                                      previous_group_affinity: *mut GROUP_AFFINITY) -> i32;
            fn GetCurrentThread() -> usize;
        }

        let group_affinity = GROUP_AFFINITY {
            mask:     1usize.checked_shl(processor.id).expect("Invalid processor ID."),
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
