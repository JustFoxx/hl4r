use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;

struct Message {
    text: String,
    debug: bool,
    save: bool,
}

struct LoggerThread {
    logger_thread: Option<JoinHandle<()>>,
}

#[derive(Clone)]
struct Logger {
    channel: Sender<Message>,
    logger_thread: Arc<LoggerThread>
}

impl Logger {
    pub fn new(name: String, mut file: File, mut debug_file: Option<File>) -> Self {
        let (sender, receiver) = channel::<Message>();

        let logger_thread = thread::spawn(move || {
            let mut handle = BufWriter::with_capacity(64 * 1024,stdout().lock());
            while let Ok(message) = receiver.recv() {
                writeln!(handle, "{}: {}", name, message.text).expect("Failed to write to stdout");
                if message.save {
                    file.write_all(message.text.as_bytes()).expect("Failed to write to log file");
                }
                if let Some(debug_file) = &mut debug_file {
                    debug_file.write_all(message.text.as_bytes()).expect("Failed to write to debug file");
                }
            }
        });

        Logger {
            channel: sender,
            logger_thread: Arc::new(LoggerThread {
                logger_thread: Some(logger_thread),
            })
        }
    }

    pub fn test() -> Self {
        let file = File::create("log.txt").unwrap();
        let debug_file = File::create("debug.txt").unwrap();
        Logger::new("test".to_string(), file, Some(debug_file))
    }

    pub fn log(&self, text: String) {
        self.channel.send(Message {
            text,
            debug: false,
            save: true,
        }).expect("Failed to send message to logger");
    }

}

impl Drop for LoggerThread {
    fn drop(&mut self) {
        if let Some(logger_thread) = self.logger_thread.take() {
            logger_thread.join().unwrap();
        }
    }
}


macro_rules! log {
    ($logger:expr, $($arg:tt)+) => {{
        $logger.log(format!($($arg)+));
    }};
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use super::*;

    #[test]
    fn it_works() {
        let logger = Logger::test();
        log!(logger,"Hello, world!");
        log!(logger,"Hello, world!");
        log!(logger,"Hello, world!");
        log!(logger,"Hello, world!");
    }
}