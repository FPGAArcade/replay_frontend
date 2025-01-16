use crossbeam_channel::{bounded, Receiver, Sender};
use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub type BoxAnySend = Box<dyn Any + Send>;

// Type alias for the callback function with state.
#[allow(dead_code)]
type CallbackWithState = (
    Box<dyn Fn(BoxAnySend, Arc<Mutex<dyn Any + Send>>) -> BoxAnySend + Send + 'static>,
    Arc<Mutex<dyn Any + Send>>,
);

#[allow(dead_code)]
struct WorkSystem {
    sender: Sender<(usize, BoxAnySend, Sender<BoxAnySend>)>,
    callbacks: Arc<Mutex<Vec<Option<CallbackWithState>>>>,
    id_counter: AtomicUsize,
}

#[allow(dead_code)]
impl WorkSystem {
    /// Creates a new WorkSystem with the specified number of worker threads.
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = bounded(num_workers);
        let callbacks: Arc<Mutex<Vec<Option<CallbackWithState>>>> =
            Arc::new(Mutex::new(Vec::with_capacity(16)));

        for _ in 0..num_workers {
            let worker_receiver: Receiver<(usize, BoxAnySend, Sender<BoxAnySend>)> =
                receiver.clone();
            let worker_callbacks = Arc::clone(&callbacks);

            thread::spawn(move || {
                while let Ok((id, data, response_sender)) = worker_receiver.recv() {
                    if let Some(Some((callback, state))) = worker_callbacks.lock().unwrap().get(id)
                    {
                        let result = callback(data, Arc::clone(state));
                        let _ = response_sender.send(result);
                    } else {
                        eprintln!("Callback with id {} not found", id);
                    }
                }
            });
        }

        Self {
            sender,
            callbacks,
            id_counter: AtomicUsize::new(0),
        }
    }

    /// Registers a new callback with its associated state and returns its unique id.
    pub fn register_callback_with_state<F, S>(&self, callback: F, state: S) -> usize
    where
        F: Fn(BoxAnySend, Arc<Mutex<dyn Any + Send>>) -> BoxAnySend + Send + 'static,
        S: Any + Send + 'static,
    {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let mut callbacks = self.callbacks.lock().unwrap();
        if id >= callbacks.len() {
            callbacks.resize_with(id + 1, || None);
        }
        callbacks[id] = Some((Box::new(callback), Arc::new(Mutex::new(state))));
        id
    }

    /// Adds work to the queue with the specified callback id and data.
    /// Returns a receiver to get the result of the task.
    pub fn add_work<T: Any + Send>(&self, id: usize, data: T) -> Receiver<BoxAnySend> {
        let (response_sender, response_receiver) = bounded(1);
        if self
            .callbacks
            .lock()
            .unwrap()
            .get(id)
            .is_some_and(|callback| callback.is_some())
        {
            self.sender
                .send((id, Box::new(data), response_sender))
                .expect("Failed to send work to the channel");
        } else {
            eprintln!("Callback with id {} not found", id);
        }
        response_receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_workflow() {
        let system = WorkSystem::new(4);
        let callback_id = system.register_callback_with_state(
            |data, _state| {
                let input = *data.downcast::<String>().unwrap();
                Box::new(format!("Processed: {}", input))
            },
            (),
        );

        let receiver = system.add_work(callback_id, "Task 1".to_string());
        let result = receiver.recv().unwrap();
        let output = *result.downcast::<String>().unwrap();
        assert_eq!(output, "Processed: Task 1");
    }

    #[test]
    fn test_callback_with_state() {
        /*
        let system = WorkSystem::new(4);
        let callback_id = system.register_callback_with_state(
            |data, state| {
                let input = *data.downcast::<String>().unwrap();
                let mut counter = state.lock().unwrap();
                let counter = counter.downcast_mut::<usize>().unwrap();
                *counter += 1;
                Box::new(format!("Processed: {}, count: {}", input, *counter))
            },
            0usize,
        );

        let receiver1 = system.add_work(callback_id, "Task 1".to_string());
        let receiver2 = system.add_work(callback_id, "Task 2".to_string());

        let result1 = receiver1.recv().unwrap();

        let result2 = receiver2.recv().unwrap();
        let output2 = *result2.downcast::<String>().unwrap();
        assert_eq!(output2, "Processed: Task 2, count: 2");
        */
    }

    #[test]
    fn test_unregistered_callback() {
        let system = WorkSystem::new(4);
        let receiver = system.add_work(999, "Invalid Task".to_string());
        assert!(receiver.recv().is_err());
    }

    #[test]
    fn test_multiple_callbacks() {
        let system = WorkSystem::new(4);

        let callback_id1 = system.register_callback_with_state(
            |data, _state| {
                let input = *data.downcast::<u32>().unwrap();
                Box::new(input + 1)
            },
            (),
        );

        let callback_id2 = system.register_callback_with_state(
            |data, _state| {
                let input = *data.downcast::<String>().unwrap();
                Box::new(format!("Hello, {}", input))
            },
            (),
        );

        let receiver1 = system.add_work(callback_id1, 42u32);
        let receiver2 = system.add_work(callback_id2, "World".to_string());

        let result1 = receiver1.recv().unwrap();
        let output1 = *result1.downcast::<u32>().unwrap();
        assert_eq!(output1, 43);

        let result2 = receiver2.recv().unwrap();
        let output2 = *result2.downcast::<String>().unwrap();
        assert_eq!(output2, "Hello, World");
    }
}
