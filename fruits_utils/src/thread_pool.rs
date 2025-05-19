use std::{
    any::Any, sync::{
        mpsc::{
            self, Receiver, Sender
        },
        Arc,
        Mutex
    }, thread::{self, JoinHandle}
};

type DefaultJob = Box<dyn FnOnce() + Send>;

pub trait Job : 'static + Send {
    fn execute(self);
}

impl Job for Box<dyn FnOnce() + Send> {
    fn execute(self) {
        self()
    }
}

enum Message<J: Job> {
    JobRequest(J),
    TerminateRequest,
}

struct Worker<J: Job = DefaultJob> {
    pub id: usize,
    pub message_receiver: Arc<Mutex<Receiver<Message<J>>>>,
    pub err_container: Arc<Mutex<Option<Box<dyn Any + Send + 'static>>>>
}

impl<J: Job> Worker<J> {
    fn run(self) -> JoinHandle<()> {
        thread::spawn(move || {
            let result = std::panic::catch_unwind(||{
                loop {
                    let message = {
                        self.message_receiver.lock().unwrap().recv().unwrap()
                    };

                    match message {
                        Message::TerminateRequest => break,
                        Message::JobRequest(job) => job.execute(),
                    }
                }
            });

            if let Err(err) = result {
                *self.err_container.lock().unwrap() = Some(err);
            }
        })
    }
}

// todo: 'static?
pub struct ThreadPool<J: Job = DefaultJob>
{
    threads: Vec<JoinHandle<()>>,
    message_sender: Sender<Message<J>>,
    err_container: Arc<Mutex<Option<Box<dyn Any + Send + 'static>>>>,
}

impl<J: Job> ThreadPool<J> {
    pub fn new(threads_count: usize) -> Self {
        assert!(threads_count > 0);

        let mut threads = Vec::with_capacity(threads_count);

        let (message_sender, message_receiver) = mpsc::channel();

        let message_receiver = Arc::new(Mutex::new(message_receiver));
        let err_container = Arc::new(Mutex::new(None));

        for id in 0..threads_count {
            let worker = Worker {
                id,
                message_receiver: Arc::clone(&message_receiver),
                err_container: Arc::clone(&err_container),
            };
            
            threads.push(worker.run());
        }
        
        Self {
            threads,
            message_sender,
            err_container,
        }
    }

    pub fn push_job(&self, job: J) {
        self.message_sender.send(Message::JobRequest(job)).unwrap();
    }

    pub fn panic_if_err(&self) {
        let err = {
            self.err_container.lock().unwrap().take()
        };

        if let Some(err) = err {
            let msg = match err.downcast_ref::<&'static str>() {
                Some(s) => *s,
                None => match err.downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<dyn Any>",
                },
            };
            
            panic!("ThreadPool worker error: {}", msg);
        }
    }
}

impl<J: Job> Drop for ThreadPool<J> {
    fn drop(&mut self) {
        for _ in 0..self.threads.len() {
            self.message_sender.send(Message::TerminateRequest).unwrap();
        }

        while let Some(join_handle) = self.threads.pop() {
            join_handle.join().unwrap();
        }
    }
}
