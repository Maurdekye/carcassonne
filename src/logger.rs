use std::{
    fs::{create_dir_all, File},
    io,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::SystemTime,
};

use std::io::Write as _;

use clap::crate_name;
use log::{Log, Record};

use crate::{util::SystemTimeExt, Args, LogLevel};

pub struct Logger {
    join_handle: Option<JoinHandle<()>>,
    message_sender: Option<Sender<MessageEvent>>,
    level_filter: log::LevelFilter,
    full_logs: bool,
}

#[derive(Clone)]
enum MessageEventKind {
    Info,
    Error,
}

#[derive(Clone)]
enum MessageEvent {
    Message(MessageEventKind, String),
    Flush,
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
                let _ = create_dir_all(&path);
                path.push(now.strftime("%H-%M-%S.log"));
                File::create(path)
            })
            .transpose()?;
        let (message_sender, message_receiver) = channel();
        let message_sender = Some(message_sender);
        let join_handle = thread::spawn(move || Logger::writer_thread(file, message_receiver));
        let join_handle = Some(join_handle);
        let level_filter = args.log_level.into();
        let full_logs = matches!(args.log_level, LogLevel::Full);
        Ok(Logger {
            join_handle,
            message_sender,
            level_filter,
            full_logs,
        })
    }

    fn writer_thread(mut file: Option<File>, message_receiver: Receiver<MessageEvent>) {
        for message in message_receiver {
            match (message, &mut file) {
                (MessageEvent::Flush, Some(file)) => file.flush().unwrap(),
                (MessageEvent::Message(kind, text), file) => {
                    match kind {
                        MessageEventKind::Info => println!("{text}"),
                        MessageEventKind::Error => eprintln!("{text}"),
                    };
                    if let Some(file) = file {
                        let _ = writeln!(file, "{text}");
                    }
                }
                _ => {}
            }
        }
    }
}

impl Log for Logger {
    fn log(&self, record: &Record) {
        use log::Level::*;
        if !self.enabled(record.metadata()) {
            return;
        }
        let timestamp = SystemTime::now().strftime("%H:%M:%S%.3f");
        let log_str = format!(
            "[{}|{}|{}{}] {}",
            record.level(),
            timestamp,
            record.target(),
            record.line().map(|x| format!(":{x}")).unwrap_or_default(),
            record.args()
        );
        let _ = self
            .message_sender
            .as_ref()
            .unwrap()
            .send(MessageEvent::Message(
                match record.level() {
                    Error | Warn => MessageEventKind::Error,
                    Info | Debug | Trace => MessageEventKind::Info,
                },
                log_str,
            ));
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        (self.full_logs || metadata.target().starts_with(crate_name!()))
            && metadata.level() <= self.level_filter
    }

    fn flush(&self) {
        let _ = self
            .message_sender
            .as_ref()
            .unwrap()
            .send(MessageEvent::Flush);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        drop(self.message_sender.take());
        self.join_handle.take().unwrap().join().unwrap();
    }
}
