use std::fmt::Display;
use std::io::Write;

// 1. What's wrong:

// 1.1. The function takes a reference to a String, but in Rust, it's more idiomatic to use a reference to a str (&str) in similar cases.
// This &str is a more general way to accept both String and string slices.
// 1.2. The function doesn't indicate whether the log was successful or not.
// 1.3. Missing unit tests.
// 1.4. Missing documentation, it's not clear how and where the log messages are filtered out.

// 2. Improvements:

// 2.1. The simplest way is to change &String to &str: pub fn write_to_log(value: &str)
// 2.2. A more advanced way is to use trait-bound AsRef<str>.
//      [Advantages] This tells us that the function can accept anything that implements AsRef<str> trait, i.e. anything that can be converted into &str.
//      [Disadvantages] Using generics trait bounds makes function harder to understand.
//
// 2.3. Change the return type of the function to Result<(), LogError>
//      [Advantages] The function returns a Result, which allows us to handle the error.
//      [Disadvantages] Handling the error might look cumbersome for logging.
// 2.3. This simplest implementation might log values into the file using the log external crate.
//
//      mod external_log {
//          pub fn write_to_log<T: AsRef<str>>(value: T) {
//              log::info!("{}", value.as_ref());
//          }
//      }
//
//      In this case we are dependent on the log crate (which is a facade) and logger implementation, i.e. env_logger.
//      Extra step with env config is required: RUST_LOG=info cargo r
//
// 2.4. The function doesn't specify the type of log, i.e. info, error, warn, debug...
//      [Advantages] Having the type of the log specified allows us to filter the log messages based on the type.
//      [Disadvantages]
//          Requires the extra step where the user specifies the type of the log.
//          More complex function signature due to additional parameters. The possible solution is to use a macro as a wrapper.
// 2.5. The function doesn't specify the distinction of log messages, i.e. file system, network, or just console.
// 2.6. It might be an option to specify multiple log types: LogType::FileSystem | LogType::Console
//      In this case the implementation requires bitmask, i.e.
//           bitflags! {
//               pub struct LogTypes: u32 {
//                   const CONSOLE = 0b0001;
//                   const FILE_SYSTEM = 0b0010;
//                   const NETWORK = 0b0100;
//               }
//           }
//       [Advantages] More options for logging.
//       [Disadvantages] The codebase requires extra dependencies. Logging might become a resource demanded in terms of CPU or Network usage.

pub enum LogType {
    Console,
    FileSystem,
    Network,
}

pub enum LogLevel {
    Info,
    Error,
    Warn,
    Debug,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Debug => write!(f, "DEBUG"),
        }
    }
}

#[derive(Debug)]
pub enum LogError {
    FileOpenError(String),
    FileWriteError(String),
    LogError(String),
}

const DEFAULT_LOG_FILE_NAME: &str = "log.txt";

/// Writes a log message to a log_type target, filtered by a log_level.
///
/// Returns `Ok(())` on success, otherwise returns LogError.
///
/// # Arguments
///
/// * `log_type` - a log target to accept the log message.
///
/// * `log_level` - a log level to filter the log message.
pub fn write_to_log<T>(log_type: LogType, log_level: LogLevel, value: T) -> Result<(), LogError>
where
    T: AsRef<str>,
{
    let log_message = format!("[{}] {}", log_level, value.as_ref());

    match log_type {
        LogType::Console => println!("{}", log_message),
        LogType::FileSystem => {
            // The file expects not to be inlined in the function, but exists outside and reused
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(DEFAULT_LOG_FILE_NAME)
                .map_err(|e| LogError::FileOpenError(e.to_string()))?;
            writeln!(file, "{}", log_message)
                .map_err(|e| LogError::FileWriteError(e.to_string()))?
        }
        LogType::Network => todo!("Requires network implementation"),
    }

    Ok(())
}

mod external_log {
    pub fn write_to_log<T>(value: T)
    where
        T: AsRef<str>,
    {
        log::info!("{}", value.as_ref());
    }
}

pub fn run() {
    let s_slice = "String slice";
    let s_owned = String::from("Owned String");

    if let Err(e) = write_to_log(
        LogType::FileSystem,
        LogLevel::Info,
        "Another one string slice",
    ) {
        eprintln!("Logging failed with error: {:?}", e);
    }
    let _ = write_to_log(LogType::Console, LogLevel::Debug, &s_owned); // Just suppress error message
    write_to_log(LogType::Console, LogLevel::Info, s_slice)
        .expect("Non-recoverable error: Logging failed");

    // Other ways to create a string in Rust. Will require more complex implementation of the write_to_log function
    // let s_pathbuf = PathBuf::from("some/path");
    // let s_path = Path::new("some/path");
    // let s_vec_of_u8 = vec!['a', 'b', 'c', 'd', 'e'];
    // let s_slice_of_u8 = &['a', 'b', 'c', 'd', 'e'];
    // let s_osstring = OsString::from("some/path");

    external_log::write_to_log(s_slice);
    external_log::write_to_log(s_owned);
    external_log::write_to_log("string slice again");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_write_to_log_to_filesystem() {
        let test_message = "Test log message";
        let log_level = LogLevel::Info;

        // 0. Remove the log file before the test
        fs::remove_file(DEFAULT_LOG_FILE_NAME).expect("Failed to delete test log file");

        // 1. Create a message
        let expected_output = format!("[{}] {}", log_level, test_message);

        // 2. Write to log using write_to_log function
        write_to_log(LogType::FileSystem, log_level, test_message).expect("Failed to write to log");

        let file = fs::File::open(DEFAULT_LOG_FILE_NAME).expect("Failed to open log file");
        let reader = BufReader::new(file);

        // Here, we assume that the last line is tho only line in the fiile
        let last_line = reader
            .lines()
            .last()
            .expect("Log file is empty")
            .expect("Failed to read from log file");

        // 3. Validate the content of the log file by comparing it with the message
        assert_eq!(last_line, expected_output);

        // Cleanup: remove the log file after the test
        fs::remove_file(DEFAULT_LOG_FILE_NAME).expect("Failed to delete test log file");
    }
}
