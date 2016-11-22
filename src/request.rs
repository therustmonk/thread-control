use std::sync::{Arc, Weak, Mutex};
use std::sync::TryLockError;
use std::time;
use std::thread;
use std::mem;

pub enum Error {
    ThreadDead,
    Timeout,
    Busy,
    WrongState,
}

enum State<I, O> {
    Free,
    Request(I),
    InProgress,
    Response(O),
}

pub fn interaction<T, O>() -> (Requester<T, O>, Responder<T, O>) {
    let arc = Arc::new(Mutex::new(State::Free));
    let weak = Arc::downgrade(&arc);
    let requester = Requester {
        data: weak,
    };
    let responder = Responder {
        data: arc,
    };
    (requester, responder)
}

#[derive(Clone)]
pub struct Requester<I, O> {
    data: Weak<Mutex<State<I, O>>>,
}

pub struct Responder<I, O> {
    data: Arc<Mutex<State<I, O>>>,
}

impl<I, O> Requester<I, O> {
    pub fn request(&self, request: I, timeout: Option<time::Duration>) -> Result<O, Error> {
        let now = time::Instant::now();
        if let Some(mutex) = self.data.upgrade() {
            match mutex.lock() {
                Ok(mut data) => {
                    if let State::Free = *data {
                        *data = State::Request(request);
                    } else {
                        return Err(Error::Busy);
                    }
                },
                Err(_) => {
                    return Err(Error::ThreadDead);
                },
            }
            loop {
                match mutex.try_lock() {
                    Ok(mut data) => {
                        let result = mem::replace(&mut*data, State::Free);
                        if let State::Response(result) = result {
                            return Ok(result);
                        } else {
                            return Err(Error::WrongState);
                        }
                    },
                    Err(TryLockError::WouldBlock) => {
                        if let Some(duration) = timeout {
                            if now.elapsed() >= duration {
                                return Err(Error::Timeout);
                            }
                        }
                    },
                    Err(TryLockError::Poisoned(_)) => {
                        return Err(Error::ThreadDead);
                    },
                }
                thread::yield_now();
            }
        } else {
            Err(Error::ThreadDead)
        }
    }
}

impl<I, O> Responder<I, O> {
    pub fn get_request(&self) -> Option<I> {
        match self.data.lock() {
            Ok(mut data) => {
                let request = mem::replace(&mut*data, State::InProgress);
                match request {
                    State::Request(input) => {
                        Some(input)
                    },
                    State::Free => {
                        mem::replace(&mut*data, State::Free);
                        None
                    },
                    State::Response(_) => {
                        // Previous result haven't processed
                        None
                    },
                    State::InProgress => {
                        panic!("It's not possible to get request if previous request haven't finished.");
                    }
                }
            },
            Err(_) => {
                None
            },
        }
    }

    pub fn set_response(&self, response: O) {
        match self.data.lock() {
            Ok(mut data) => {
                let request = mem::replace(&mut*data, State::InProgress);
                match request {
                    State::InProgress | State::Request(_) => {
                        mem::replace(&mut*data, State::Response(response));
                    },
                    State::Response(_) => {
                        panic!("Impossible to set response twice.");
                    },
                    State::Free => {
                        panic!("Trying to set response to nothing.");
                    }
                }
            },
            Err(_) => {
            },
        }
    }
}
