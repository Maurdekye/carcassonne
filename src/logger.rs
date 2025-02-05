use std::{
    fs::File,
    io,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::SystemTime,
};

use std::io::Write as _;

use log::{Log, Record};

use crate::{util::SystemTimeExt, Args};

pub struct Logger {
    args: Args,
    join_handle: Option<JoinHandle<()>>,
    message_sender: Option<Sender<Option<String>>>,
}

impl Logger {
    pub fn new(args: Args) -> Result<Logger, io::Error> {
        let file = args
            .save_logs
            .clone()
            .flatten()
            .map(|mut path| {
                let now = SystemTime::now();
                path.push(now.strftime("%Y-%m-%d"));
                path.push(now.strftime("%H-%M-%S.log"));
                Ok::<_, io::Error>(File::create(path)?)
            })
            .transpose()?;
        let (message_sender, message_receiver) = channel();
        let message_sender = Some(message_sender);
        let join_handle = thread::spawn(move || Logger::writer_thread(file, message_receiver));
        let join_handle = Some(join_handle);
        Ok(Logger {
            args,
            join_handle,
            message_sender,
        })
    }

    fn writer_thread(mut file: Option<File>, message_receiver: Receiver<Option<String>>) {
        for message in message_receiver {
            match (message, &mut file) {
                (None, Some(file)) => file.flush().unwrap(),
                (Some(message), file) => {
                    println!("{message}");
                    if let Some(file) = file {
                        let _ = writeln!(file, "{message}");
                    }
                }
                _ => {}
            }
        }
    }
}

impl Log for Logger {
    fn log(&self, record: &Record) {
        let timestamp = SystemTime::now().strftime("$H:$M:$S$.3f");
        let log_str = format!("[{}|{}] {}", record.level(), timestamp, record.args());
        let _ = self.message_sender.as_ref().unwrap().send(Some(log_str));
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.args.log_level
    }

    fn flush(&self) {
        let _ = self.message_sender.as_ref().unwrap().send(None);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        drop(self.message_sender.take());
        self.join_handle.take().unwrap().join().unwrap();
    }
}
