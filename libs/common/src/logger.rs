use crate::util::file_exists;
use log::{debug, LevelFilter, SetLoggerError};
use log4rs::{
  append::{
    console::{ConsoleAppender, Target},
    file::FileAppender,
  },
  config::{Appender, Config, Root},
  encode::pattern::PatternEncoder,
  filter::threshold::ThresholdFilter,
};
use std::fs;

pub fn init_log4rs(default_log_file_path: &String, default_log_level: &String, default_log_file_level: &String) -> Result<(), SetLoggerError> {
  // closure to get LevelFilter from env string
  let get_log_level = |env_level: String| -> LevelFilter {
    match env_level.to_uppercase().as_str() {
      "OFF" => LevelFilter::Off,
      "ERROR" => LevelFilter::Error,
      "WARN" => LevelFilter::Warn,
      "INFO" => LevelFilter::Info,
      "DEBUG" => LevelFilter::Debug,
      "TRACE" => LevelFilter::Trace,
      _ => LevelFilter::Error,
    }
  };
  let log_level = get_log_level(default_log_level.into());
  let logfile_level = get_log_level(default_log_file_level.into());
  // always delete old log
  if file_exists(default_log_file_path.as_str()) {
    debug!("removing old log file:{}", default_log_file_path);
    fs::remove_file(default_log_file_path.clone()).unwrap();
  };
  // Build a stderr logger.
  let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
  // Logging to log file.
  let logfile = FileAppender::builder()
    // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
    .encoder(Box::new(PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)}] {l} - {m}\n")))
    .build(default_log_file_path)
    .unwrap();
  // Log Trace level output to file where trace is the default level
  // and the programmatically specified level to stderr.
  let config = Config::builder()
    .appender(Appender::builder().build("logfile", Box::new(logfile)))
    .appender(Appender::builder().filter(Box::new(ThresholdFilter::new(log_level))).build("stderr", Box::new(stderr)))
    .build(Root::builder().appender("logfile").appender("stderr").build(logfile_level))
    .unwrap();

  // Use this to change log levels at runtime.
  // This means you can change the default log level to trace
  // if you are trying to debug an issue and need more logs on then turn it off
  // once you are done.
  let _handle = log4rs::init_config(config)?;

  // error!("Goes to stderr and file");
  // warn!("Goes to stderr and file");
  // info!("Goes to stderr and file");
  // debug!("Goes to file only");
  // trace!("Goes to file only");
  // debug!("current log level: '{:?}'", log_level);

  Ok(())
}
