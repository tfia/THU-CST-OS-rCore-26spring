use alloc::vec::Vec;
use alloc::vec;

use crate::sync::UPSafeCell;

/// Deadlock detector structure
pub struct DeadlockDetector {
    /// Deadlock detector inner
    pub inner: UPSafeCell<DeadlockDetectorInner>,
}

pub struct DeadlockDetectorInner {
    pub available: Vec<usize>,
    pub allocation: Vec<Vec<usize>>,
    pub need: Vec<Vec<usize>>,
}

impl DeadlockDetectorInner {
    /// Check if the system is in a deadlock state, 
    /// return true if there is a deadlock, otherwise return false
    pub fn check(&self) -> bool {
        trace!("kernel: DeadlockDetectorInner::check");
        let mut work = self.available.clone();
        let mut finish = vec![false; self.allocation.len()];

        loop {
            let mut found = false;
            for i in 0..self.allocation.len() {
                // Finish[i] == false; && Need[i,j] <= Work[j];
                if !finish[i] && self.need[i].iter().zip(&work).all(|(n, w)| n <= w) {
                    // Work[j] = Work[j] + Allocation[i, j];
                    for j in 0..work.len() {
                        work[j] += self.allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }
            if !found {
                break;
            }
        }

        !finish.iter().all(|&f| f)
    }
}

impl DeadlockDetector {
    /// Create a new deadlock detector
    pub fn new() -> Self {
        trace!("kernel: DeadlockDetector::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(DeadlockDetectorInner {
                    available: Vec::new(),
                    allocation: Vec::new(),
                    need: Vec::new(),
                })
            },
        }
    }

    /// Add a new thread with thread id
    pub fn add_thread(&self, tid: usize) {
        trace!("kernel: DeadlockDetector::add_thread");
        let mut inner = self.inner.exclusive_access();
        let available_len = inner.available.len();
        if inner.allocation.len() <= tid {
            inner.allocation.resize(tid + 1, vec![0; available_len]);
            inner.need.resize(tid + 1, vec![0; available_len]);
        } 
        // else {
        //     inner.allocation[tid] = vec![0; available_len];
        //     inner.need[tid] = vec![0; available_len];
        // }
    }

    /// Add a new resource with resource id and count
    pub fn add_resource(&self, rid: usize, count: usize) {
        let mut inner = self.inner.exclusive_access();
        if inner.available.len() <= rid {
            inner.available.resize(rid + 1, 0);
            inner.available[rid] = count;
            for i in 0..inner.allocation.len() {
                inner.allocation[i].resize(rid + 1, 0);
                inner.need[i].resize(rid + 1, 0);
            }
        } else {
            inner.available[rid] = count;
            for i in 0..inner.allocation.len() {
                inner.allocation[i][rid] = 0;
                inner.need[i][rid] = 0;
            }
        }
    }

    /// Claim the need of a thread for a resource
    pub fn claim(&self, tid: usize, rid: usize, count: usize) {
        trace!("kernel: DeadlockDetector::claim tid[{}] rid[{}] count[{}]", tid, rid, count);
        let mut inner = self.inner.exclusive_access();
        // assert!(inner.need.len() <= tid || inner.need[tid].len() <= rid);
        inner.need[tid][rid] += count;
    }

    /// Revert the claim of a thread for a resource
    pub fn unclaim(&self, tid: usize, rid: usize, count: usize) {
        trace!("kernel: DeadlockDetector::unclaim tid[{}] rid[{}] count[{}]", tid, rid, count);
        let mut inner = self.inner.exclusive_access();
        // assert!(inner.need.len() <= tid || inner.need[tid].len() <= rid);
        inner.need[tid][rid] -= count;
    }

    /// Try to request resources for a thread
    pub fn request(&self, tid: usize, rid: usize, count: usize) -> bool {
        trace!("kernel: DeadlockDetector::request tid[{}] rid[{}] count[{}]", tid, rid, count);
        let mut inner = self.inner.exclusive_access();
        if count > inner.available[rid] || count > inner.need[tid][rid] {
            return false;
        }
        inner.available[rid] -= count;
        inner.allocation[tid][rid] += count;
        inner.need[tid][rid] -= count;
        
        if inner.check() { // deadlock occurs
            inner.available[rid] += count;
            inner.allocation[tid][rid] -= count;
            inner.need[tid][rid] += count;
            false
        } else {
            true
        }
    }

    /// Release resources for a thread
    pub fn release(&self, tid: usize, rid: usize, count: usize) {
        trace!("kernel: DeadlockDetector::release tid[{}] rid[{}] count[{}]", tid, rid, count);
        let mut inner = self.inner.exclusive_access();
        inner.available[rid] += count;
        inner.allocation[tid][rid] -= count;
    }
}