mod processors;

use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, Condvar, Barrier};
use std::sync::atomic::{AtomicUsize, Ordering};

type Coordinate            = usize;
type WorkItem              = PixelRange;
type WorkCallback<C, P, L> = dyn Fn(&C, &mut L, Coordinate, &mut [P]) + 'static + Send + Sync;

#[derive(Copy, Clone)]
struct PixelRange {
    start: Coordinate,
    size:  Coordinate,
}

struct AtomicQueue<T> {
    queue: Vec<T>,
    index: AtomicUsize,
}

impl<T: Copy> AtomicQueue<T> {
    fn new(queue: Vec<T>) -> Self {
        Self {
            queue,
            index: AtomicUsize::new(0),
        }
    }

    fn pop(&self) -> Option<T> {
        let index = self.index.fetch_add(1, Ordering::SeqCst);

        if index < self.queue.len() {
            Some(self.queue[index])
        } else {
            None
        }
    }
}

struct Work<C, P, Cb: ?Sized> {
    queue:       AtomicQueue<WorkItem>,
    context:     *const C,
    buffer:      *mut P,
    buffer_size: usize,
    callback:    Cb,
}

unsafe impl<C: Send, P: Send, Cb: ?Sized + Send> Send for Work<C, P, Cb> {}
unsafe impl<C: Sync, P: Send, Cb: ?Sized + Sync> Sync for Work<C, P, Cb> {}

type DynWork<C, P, L> = Work<C, P, WorkCallback<C, P, L>>;

struct State<C, P, L> {
    work:    Mutex<Option<Arc<DynWork<C, P, L>>>>,
    work_cv: Condvar,
    barrier: Barrier,
}

pub struct ParallelRenderer<C, P, L> {
    state:   Arc<State<C, P, L>>,
    threads: Vec<JoinHandle<()>>,
    done_rx: Receiver<()>,
    counter: Option<Arc<()>>,
}

impl<C, P, L> ParallelRenderer<C, P, L> 
    where C: 'static + Send + Sync,
          P: 'static + Send + Sync + Copy,
          L: 'static + Default,
{
    pub fn new() -> Self {
        let processors   = processors::logical();
        let thread_count = processors.len();

        let state = Arc::new(State {
            work:    Mutex::new(None),
            work_cv: Condvar::new(),
            barrier: Barrier::new(thread_count),
        });

        let mut threads = Vec::with_capacity(thread_count);

        let (done_tx, done_rx) = mpsc::channel();
        let counter            = Arc::new(());

        for (tid, processor) in processors.into_iter().enumerate() {
            let state   = state.clone();
            let done_tx = done_tx.clone();
            let counter = counter.clone();

            threads.push(thread::spawn(move || {
                processors::pin_to_processor(&processor, false);

                state.barrier.wait();

                while Arc::strong_count(&counter) > thread_count {
                    let mut work = state.work.lock().unwrap();

                    while work.is_none() {
                        if Arc::strong_count(&counter) <= thread_count {
                            return;
                        }

                        work = state.work_cv.wait(work).unwrap();
                    }

                    let work: Arc<DynWork<C, P, L>> = {
                        let result = work.as_ref().unwrap().clone();

                        drop(work);

                        result
                    };

                    state.barrier.wait();

                    if tid == 0 {
                        *state.work.lock().unwrap() = None;
                    }

                    let context   = unsafe { &*work.context };
                    let mut local = L::default();

                    while let Some(item) = work.queue.pop() {
                        let buffer = unsafe {
                            assert!(work.buffer_size >=
                                    item.start.checked_add(item.size).unwrap());

                            let buffer: *mut P = work.buffer;

                            std::slice::from_raw_parts_mut(buffer.add(item.start),
                                                           item.size)
                        };

                        (work.callback)(context, &mut local, item.start, buffer);
                    }

                    done_tx.send(()).unwrap();

                    state.barrier.wait();
                }
            }));
        }

        Self {
            counter: Some(counter),
            state,
            threads,
            done_rx,
        }
    }

    pub fn render<F>(&mut self, context: &C, buffer: &mut [P], callback: F)
        where F: Fn(&C, &mut L, Coordinate, &mut [P]) + 'static + Send + Sync
    {
        let pixel_count = buffer.len();

        {
            let pixel_queue = {
                let range_count      = self.threads.len() * 64;
                let pixels_per_range = (pixel_count + range_count - 1) / range_count;

                let mut ranges = Vec::with_capacity(range_count);

                for idx in 0..range_count {
                    let start = pixels_per_range * idx;

                    let outside_screen = if idx + 1 == range_count {
                        (pixels_per_range * range_count).checked_sub(pixel_count).unwrap()
                    } else {
                        0
                    };

                    let size = pixels_per_range.checked_sub(outside_screen).unwrap();

                    ranges.push(PixelRange {
                        start,
                        size,
                    });
                }

                AtomicQueue::new(ranges)
            };

            let mut work = self.state.work.lock().unwrap();

            *work = Some(Arc::new(Work {
                callback,
                context,
                queue:       pixel_queue,
                buffer:      buffer.as_mut_ptr(),
                buffer_size: buffer.len(),
            }));

            self.state.work_cv.notify_all();
        }

        for _ in 0..self.threads.len() {
            let _ = self.done_rx.recv().unwrap();
        }
    }
}

impl<C, P, L> Drop for ParallelRenderer<C, P, L> {
    fn drop(&mut self) {
        drop(self.counter.take().unwrap());

        self.state.work_cv.notify_all();

        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }
}
