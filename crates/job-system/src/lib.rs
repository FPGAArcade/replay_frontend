//! A simple job system for parallel task execution
//!
//! This library provides a thread-based job system for executing tasks in parallel.
//! Jobs can share state through Arc<Mutex<T>> and return results through channels.

use crossbeam_channel::{bounded, Receiver, Sender};
use std::any::Any;
use std::thread;
use thiserror::Error;

/// Errors that can occur during job execution
#[derive(Error, Debug)]
pub enum JobError {
    #[error("Failed to downcast type: expected {expected}")]
    DowncastError { expected: &'static str },

    #[error("Channel send error: {0}")]
    ChannelSendError(String),

    #[error("Channel receive error: {0}")]
    ChannelReceiveError(String),

    #[error("Lock acquisition failed")]
    LockError,

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Network operation failed: {0}")]
    NetworkError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for job operations
pub type JobResult<T> = Result<T, JobError>;

/// Type for passing arbitrary data between jobs
pub type BoxAnySend = Box<dyn Any + Send>;

/// Internal type for job functions
type Job = Box<dyn FnOnce(BoxAnySend) -> JobResult<BoxAnySend> + Send + 'static>;

/// Handle to a scheduled job
pub struct JobHandle {
    pub receiver: Receiver<JobResult<BoxAnySend>>,
}

impl JobHandle {
    /// Check if the job has completed without blocking
    pub fn is_finished(&self) -> bool {
        self.receiver.is_empty() == false
    }

    /// Wait for the job to complete and get its result with automatic type conversion
    pub fn get_result<T: 'static>(self) -> JobResult<T> {
        let result = self
            .receiver
            .recv()
            .map_err(|e| JobError::ChannelReceiveError(e.to_string()))?;

        match result {
            Ok(data) => data
                .downcast()
                .map(|boxed| *boxed)
                .map_err(|_| JobError::DowncastError {
                    expected: std::any::type_name::<T>(),
                }),
            Err(e) => Err(e),
        }
    }

    /// Try to get the job's result without blocking, with automatic type conversion
    pub fn try_get_result<T: 'static>(&self) -> Option<JobResult<T>> {
        self.receiver.try_recv().ok().map(|result| match result {
            Ok(data) => data
                .downcast()
                .map(|boxed| *boxed)
                .map_err(|_| JobError::DowncastError {
                    expected: std::any::type_name::<T>(),
                }),
            Err(e) => Err(e),
        })
    }

    /// Get the raw result without type conversion
    pub fn get_result_raw(self) -> JobResult<BoxAnySend> {
        self.receiver
            .recv()
            .map_err(|e| JobError::ChannelReceiveError(e.to_string()))?
    }
}

/// Main job system for managing parallel task execution
pub struct JobSystem {
    sender: Sender<(Option<Job>, BoxAnySend, Sender<JobResult<BoxAnySend>>)>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl JobSystem {
    /// Creates a new JobSystem with the specified number of worker threads
    ///
    /// # Arguments
    /// * `num_threads` - Number of worker threads to create
    ///
    /// # Errors
    /// Returns `JobError` if thread creation fails
    pub fn new(num_threads: usize) -> JobResult<Self> {
        let (sender, receiver) = bounded(32);
        let receiver_clone: Receiver<(Option<Job>, BoxAnySend, Sender<JobResult<BoxAnySend>>)> =
            receiver.clone();

        let mut handles = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let receiver = receiver_clone.clone();
            let handle = thread::spawn(move || {
                while let Ok((job, data, result_sender)) = receiver.recv() {
                    match job {
                        Some(job) => {
                            let result = job(data);
                            let _ = result_sender.send(result);
                        }
                        None => break,
                    }
                }
            });
            handles.push(handle);
        }

        Ok(JobSystem { sender, handles })
    }

    /// Schedules a job for execution and returns a handle to track its progress
    ///
    /// # Arguments
    /// * `f` - Job function to execute
    /// * `data` - Data to pass to the job
    ///
    /// # Errors
    /// Returns `JobError` if job scheduling fails
    pub fn schedule_job<F>(&self, f: F, data: BoxAnySend) -> JobResult<JobHandle>
    where
        F: FnOnce(BoxAnySend) -> JobResult<BoxAnySend> + Send + 'static,
    {
        let (result_sender, result_receiver) = bounded(1);
        let job = Box::new(f) as Job;

        self.sender
            .send((Some(job), data, result_sender))
            .map_err(|e| JobError::ChannelSendError(e.to_string()))?;

        Ok(JobHandle {
            receiver: result_receiver,
        })
    }
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        for _ in 0..self.handles.len() {
            let (sender, _) = bounded(1);
            let _ = self.sender.send((None, Box::new(()), sender));
        }

        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_basic_job() -> JobResult<()> {
        let job_system = JobSystem::new(1)?;

        let result = Arc::new(AtomicI32::new(0));
        let result_clone = result.clone();

        let handle = job_system.schedule_job(
            move |data: BoxAnySend| {
                let number = *data
                    .downcast::<i32>()
                    .map_err(|_| JobError::DowncastError { expected: "i32" })?;
                result_clone.store(number * 2, Ordering::SeqCst);
                Ok(Box::new(()))
            },
            Box::new(21),
        )?;

        let _: () = handle.get_result()?;
        assert_eq!(result.load(Ordering::SeqCst), 42);
        Ok(())
    }

    #[test]
    fn test_multiple_jobs() -> JobResult<()> {
        let job_system = JobSystem::new(2)?;
        let counter = Arc::new(AtomicI32::new(0));
        let mut handles = Vec::new();

        // Schedule 5 jobs
        for _ in 0..5 {
            let counter = counter.clone();
            let handle = job_system.schedule_job(
                move |_: BoxAnySend| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(Box::new(()))
                },
                Box::new(()),
            )?;
            handles.push(handle);
        }

        // Wait for all results
        for handle in handles {
            let _: () = handle.get_result()?;
        }

        assert_eq!(counter.load(Ordering::SeqCst), 5);
        Ok(())
    }

    #[test]
    fn test_job_status() -> JobResult<()> {
        let job_system = JobSystem::new(1)?;

        let handle = job_system.schedule_job(
            move |_: BoxAnySend| {
                thread::sleep(Duration::from_millis(100));
                Ok(Box::new(42))
            },
            Box::new(()),
        )?;

        // Job should not be finished immediately
        assert!(!handle.is_finished());
        assert!(handle.try_get_result::<i32>().is_none());

        // Wait for result
        let result: i32 = handle.get_result()?;
        assert_eq!(result, 42);

        Ok(())
    }

    #[test]
    fn test_file_not_found() -> JobResult<()> {
        let job_system = JobSystem::new(1)?;

        // Static job function that tries to open a non-existent file
        fn file_opening_job(data: BoxAnySend) -> JobResult<BoxAnySend> {
            let file_path = data.downcast::<String>().unwrap();

            match File::open(&*file_path) {
                Ok(_) => Ok(Box::new(true) as BoxAnySend),
                Err(e) => Err(JobError::FileError(e.to_string())),
            }
        }

        let handle = job_system.schedule_job(
            file_opening_job,
            Box::new("non_existent_file.txt".to_string()),
        )?;

        // Verify we get back the file error
        match handle.get_result::<bool>() {
            Err(JobError::FileError(msg)) => {
                assert!(msg.contains("No such file"));
                Ok(())
            }
            other => panic!("Expected FileError, got {:?}", other),
        }
    }
}
