use crossbeam_channel::bounded;
use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use thiserror::Error;

pub use crossbeam_channel::{Receiver, Sender};

// Type alias for Box<dyn Any + Send>
pub type BoxAnySend = Box<dyn Any + Send>;
pub type AnySend = dyn Any + Send;

// Define structured errors using thiserror
#[derive(Error, Debug)]
pub enum CallbackError {
    #[error("Invalid data type provided")]
    InvalidDataType,

    #[error("Invalid state type provided")]
    InvalidStateType,

    #[error("Callback with id {0} not found")]
    CallbackNotFound(usize),

    #[error("Other error: {0}")]
    Other(String),
}

pub type WorkerResult = Result<BoxAnySend, CallbackError>;

// Type alias for the callback function with state.
type CallbackWithState = (
    Box<
        dyn Fn(BoxAnySend, Arc<Mutex<AnySend>>) -> Result<BoxAnySend, CallbackError>
            + Send
            + 'static,
    >,
    Arc<Mutex<AnySend>>,
);

#[allow(clippy::type_complexity)]
pub struct WorkSystem {
    sender: Sender<(usize, BoxAnySend, Sender<Result<BoxAnySend, CallbackError>>)>,
    callbacks: Arc<Mutex<Vec<Option<CallbackWithState>>>>,
    id_counter: AtomicUsize,
}

impl WorkSystem {
    #[allow(clippy::type_complexity)]
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = bounded(num_workers);
        let callbacks: Arc<Mutex<Vec<Option<CallbackWithState>>>> =
            Arc::new(Mutex::new(Vec::new()));

        for i in 0..num_workers {
            let worker_receiver: Receiver<(
                usize,
                BoxAnySend,
                Sender<Result<BoxAnySend, CallbackError>>,
            )> = receiver.clone();
            let worker_callbacks = Arc::clone(&callbacks);

            let name = format!("background_worker_{}", i);

            let _ = thread::Builder::new()
                .name(name.to_owned())
                .spawn(move || {
                while let Ok((id, data, response_sender)) = worker_receiver.recv() {
                    if let Some(Some((callback, state))) = worker_callbacks.lock().unwrap().get(id)
                    {
                        let result = callback(data, Arc::clone(state));
                        let _ = response_sender.send(result);
                    } else {
                        let _ = response_sender.send(Err(CallbackError::CallbackNotFound(id)));
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

    pub fn register_callback_with_state<F>(&self, callback: F, state: Arc<Mutex<AnySend>>) -> usize
    where
        F: Fn(BoxAnySend, Arc<Mutex<AnySend>>) -> Result<BoxAnySend, CallbackError>
            + Send
            + 'static,
    {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let mut callbacks = self.callbacks.lock().unwrap();
        if id >= callbacks.len() {
            callbacks.resize_with(id + 1, || None);
        }
        callbacks[id] = Some((Box::new(callback), state));
        id
    }

    pub fn add_work<T: Any + Send>(
        &self,
        id: usize,
        data: T,
    ) -> Receiver<Result<BoxAnySend, CallbackError>> {
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
            let _ = response_sender.send(Err(CallbackError::CallbackNotFound(id)));
        }
        response_receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_callback_with_state() {
        let system = WorkSystem::new(4);

        let state: Arc<Mutex<AnySend>> = Arc::new(Mutex::new(0usize));

        let callback_id = system.register_callback_with_state(
            |data, state| {
                let input = *data
                    .downcast::<usize>()
                    .map_err(|_| CallbackError::InvalidDataType)?;
                let mut counter = state.lock().unwrap();
                let counter = counter
                    .downcast_mut::<usize>()
                    .ok_or(CallbackError::InvalidStateType)?;
                *counter += input;
                Ok(Box::new(*counter))
            },
            state.clone(),
        );

        let receiver1 = system.add_work(callback_id, 5usize);
        let receiver2 = system.add_work(callback_id, 10usize);

        let result1 = receiver1.recv().unwrap().unwrap();
        let result2 = receiver2.recv().unwrap().unwrap();

        assert_eq!(*result1.downcast::<usize>().unwrap(), 5);
        assert_eq!(*result2.downcast::<usize>().unwrap(), 15);

        let final_state = state.lock().unwrap();
        let final_value = final_state.downcast_ref::<usize>().unwrap();
        assert_eq!(*final_value, 15);
    }

    #[test]
    fn test_callback_with_string_concatenation() {
        let system = WorkSystem::new(4);

        let state: Arc<Mutex<AnySend>> = Arc::new(Mutex::new(String::new()));

        let callback_id = system.register_callback_with_state(
            |data, state| {
                let input = *data
                    .downcast::<String>()
                    .map_err(|_| CallbackError::InvalidDataType)?;
                let mut state = state.lock().unwrap();
                let state = state
                    .downcast_mut::<String>()
                    .ok_or(CallbackError::InvalidStateType)?;
                state.push_str(&input);
                Ok(Box::new(state.clone()))
            },
            state.clone(),
        );

        let receiver1 = system.add_work(callback_id, "Hello, ".to_string());
        let receiver2 = system.add_work(callback_id, "world!".to_string());

        let result1 = receiver1.recv().unwrap().unwrap();
        let result2 = receiver2.recv().unwrap().unwrap();

        assert_eq!(*result1.downcast::<String>().unwrap(), "Hello, ");
        assert_eq!(*result2.downcast::<String>().unwrap(), "Hello, world!");

        let final_state = state.lock().unwrap();
        let final_value = final_state.downcast_ref::<String>().unwrap();
        assert_eq!(*final_value, "Hello, world!");
    }

    #[test]
    fn test_invalid_data_type() {
        let system = WorkSystem::new(4);

        let state: Arc<Mutex<AnySend>> = Arc::new(Mutex::new(0usize));

        let callback_id = system.register_callback_with_state(
            |data, _state| {
                data.downcast::<String>()
                    .map_err(|_| CallbackError::InvalidDataType)?;
                Ok(Box::new(()))
            },
            state,
        );

        let receiver = system.add_work(callback_id, 42usize);
        let result = receiver.recv().unwrap();

        assert!(matches!(result, Err(CallbackError::InvalidDataType)));
    }

    #[test]
    fn test_callback_not_found() {
        let system = WorkSystem::new(4);

        let receiver = system.add_work(999, "Test data".to_string());
        let result = receiver.recv().unwrap();

        assert!(matches!(result, Err(CallbackError::CallbackNotFound(999))));
    }
}
