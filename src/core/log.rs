//! A lightweight logger.
//!
//! The `Logger` provides a single logging API that abstracts over the
//! actual log sinking implementation.
//!
//! A log request consists of a _level_ and a _message_.
//!
//! # Use
//!
//! The basic logging of the `Logger` is through the following macros:
//! [`emerg`], [`alert`], [`crit`], [`error`], [`warn`], [`notice`], [`debug`]
//! where `emerg!` represents the highest-priority log messages
//! and `debug!` thw lowest.
//!
//! The log messages are filtered by configuring the log level to exclude message with lower priority.
//!
//!
//! - [`emerg!`](../../macro.emerg.html)
//! - [`alert!`](../../macro.alert.html)
//! - [`crit!`](../../macro.crit.html)
//! - [`error!`](../../macro.error.html)
//! - [`warn!`](../../macro.warn.html)
//! - [`notice!`](../../macro.notice.html)
//! - [`info!`](../../macro.info.html)
//! - [`debug!`](../../macro.debug.html)
//!
//!
//! ## Example
//!
//! ```
//! #[macro_use]
//! extern crate dpdk;
//!
//! # #[derive(Debug)] pub struct Yak(String);
//! # impl Yak { fn shave(&mut self, _: u32) {} }
//! # fn find_a_razor() -> Result<u32, u32> { Ok(1) }
//!
//! use dpdk::core::log;
//! use std::io;
//!
//! pub fn shave_the_yak(yak: &mut Yak) {
//!     let mut stdout = io::stdout();
//!     let mut logger = log::Logger::new(log::Level::Info, &mut stdout);
//!
//!     loop {
//!         match find_a_razor() {
//!             Ok(razor) => {
//!                 info!(logger, "Razor located: {}", razor);
//!                 yak.shave(razor);
//!                 break;
//!             }
//!             Err(e) => {
//!                 warn!(logger, "Unable to locate a razor: {}, retrying", e);
//!             }
//!         }
//!     }
//!
//! }
//!
//! # fn main() {}
//! ```
//!
//! # Log format
//!
//! Every log message obeys the following fixed and easily-parsable format:
//! ```text
//! <YYYY>-<mm>-<dd> <HH>:<MM>:<SS>.<mss> [<level>] <file>:<line> <message>\n
//! ```
//!
//! - `<YYYY>` denotes years to *4* digits, any message having year larger than `9999` will be ignored.
//! - `<mm>` denotes minutes zero-padded to *2* digits,
//! - `<dd>` denotes days zero-padded to *2* digits,
//! - `<HH>` denotes hours zero-padded to *2* digits,
//! - `<MM>` denotes minutes zero-padded to *2* digits,
//! - `<SS>` denotes seconds zero-padded to *2* digits,
//! - `<mss>` denotes milliseconds zero-padded to *3* digits.
//! - `<level>` is the log level as defined by `Level`.
//! - `<message>` is the log message.
//!
//! NOTE: a newline is automatically inserted at the end.
//!
//!
//! # Errors
//!
//! Any errors returned by the sink when writing are ignored.
//!

use std::io;
use std::fmt;
use std::error;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// The standard logging macro.
///
/// This macro will generically log with the specified `Level` and arguments
/// list.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// log!(logger, log::Level::Error, "{}\n", 123);
/// # }
/// ```
#[macro_export]
macro_rules! log {
    ($logger:expr, $lvl:expr, $($arg:tt)+) => {
        $logger.log($lvl, file!(), line!(), format_args!($($arg)+));
    };
}

/// Logs a message at the emerg level.
///
/// # Example
/// 
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// emerg!(logger, "{}\n", "This is an emerg level message");
/// # }
/// ```
#[macro_export]
macro_rules! emerg {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Emerg, $($arg)+);
    };
}

/// Logs a message at the alert level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// alert!(logger, "{}\n", "This is an alert level message");
/// # }
/// ```
#[macro_export]
macro_rules! alert {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Alert, $($arg)+);
    };
}

/// Logs a message at the crit level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// crit!(logger, "{}\n", "This is a crit level message");
/// # }
/// ```
#[macro_export]
macro_rules! crit {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Crit, $($arg)+);
    };
}

/// Logs a message at the error level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// error!(logger, "{}\n", "This is an error level message");
/// # }
/// ```
#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Error, $($arg)+);
    };
}

/// Logs a message at the warn level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// warn!(logger, "{}\n", "This is a warn level message");
/// # }
/// ```
#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Warn, $($arg)+);
    };
}

/// Logs a message at the notice level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// notice!(logger, "{}\n", "This is a notice level message");
/// # }
/// ```
#[macro_export]
macro_rules! notice {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Notice, $($arg)+);
    };
}

/// Logs a message at the info level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// info!(logger, "{}\n", "This is a info level message");
/// # }
/// ```
#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Info, $($arg)+);
    };
}

/// Logs a message at the debug level.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate dpdk;
///
/// use dpdk::core::log;
/// use std::io;
///
/// # fn main() {
/// let mut stderr = io::stderr();
/// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
///
/// debug!(logger, "{}\n", "This is a debug level message");
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt)+) => {
        log!($logger, $crate::core::log::Level::Debug, $($arg)+);
    };
}


/// A format optimized Logger
pub struct Logger<'a> {
    filter: Level,
    writer: &'a mut io::Write,
    last_ts: u64,
    buffer: [u8; 4096],
}

impl<'a> Logger<'a> {
    /// Constructs a Logger instance with specified `Level` and
    /// io backend writer
    ///
    /// ```
    /// #[macro_use]
    /// extern crate dpdk;
    ///
    /// use dpdk::core::log;
    /// use std::io;
    ///
    /// # fn main() {
    /// let mut stderr = io::stdout();
    /// let mut logger = log::Logger::new(log::Level::Debug, &mut stderr);
    /// # }
    /// ```
    #[inline]
    pub fn new(filter: Level, writer: &'a mut io::Write) -> Self {
        Self {
            filter: filter,
            writer: writer,
            last_ts: 0,
            buffer: [0u8; 4096],
        }
    }

    /// Sets the logger log level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.filter = level;
    }

    /// Logs the message.
    pub fn log<'r>(&mut self, level: Level, file: &'static str, mut line: u32,
                   args: fmt::Arguments<'r>) {
        if level > self.filter {
            return;
        }

        let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let secs_since_epoch = dur.as_secs();
        let msec = dur.subsec_millis();

        if secs_since_epoch >= 253402300800u64 {
            panic!("can't format year 9999");
        }

        let (year, mon, day, hr, min, sec) = private::localtime(secs_since_epoch);

        if self.last_ts != secs_since_epoch {
            self.last_ts = secs_since_epoch;

            self.buffer[0] = b'0' + (year / 1000) as u8;
            self.buffer[1] = b'0' + (year / 100 % 10) as u8;
            self.buffer[2] = b'0' + (year / 10 % 100) as u8;
            self.buffer[3] = b'0' + (year % 10) as u8;
            self.buffer[4] = b'-';
            self.buffer[5] = b'0' + (mon / 10) as u8;
            self.buffer[6] = b'0' + (mon % 10) as u8;
            self.buffer[7] = b'-';
            self.buffer[8] = b'0' + (day / 10) as u8;
            self.buffer[9] = b'0' + (day % 10) as u8;
            self.buffer[10] = b' ';
            self.buffer[11] = b'0' + (hr / 10) as u8;
            self.buffer[12] = b'0' + (hr % 10) as u8;
            self.buffer[13] = b':';
            self.buffer[14] = b'0' + (min / 10) as u8;
            self.buffer[15] = b'0' + (min % 10) as u8;
            self.buffer[16] = b':';
            self.buffer[17] = b'0' + (sec / 10) as u8;
            self.buffer[18] = b'0' + (sec % 10) as u8;
            self.buffer[19] = b'.';
        }

        self.buffer[20] = b'0' + (msec / 100) as u8;
        self.buffer[21] = b'0' + (msec / 10 % 10) as u8;
        self.buffer[22] = b'0' + (msec % 10) as u8;
        self.buffer[23] = b' ';

        self.buffer[24] = b'[';

        let len = LOG_LEVEL_NAMES[level as usize].len();
        self.buffer[25..][..len].copy_from_slice(LOG_LEVEL_NAMES[level as usize].as_bytes());

        let mut idx = 25 + len;

        self.buffer[idx] = b']';
        idx += 1;

        self.buffer[idx] = b' ';
        idx += 1;

        self.buffer[idx..][..file.len()].copy_from_slice(file.as_bytes());
        idx += file.len();

        self.buffer[idx] = b':';
        idx += 1;

        let digits_begin = idx;
        const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
        loop {
            self.buffer[idx] = DIGITS[(line % 10) as usize];
            idx += 1;

            line /= 10;
            if line == 0 {
                break;
            }
        }
        self.buffer[digits_begin..idx].reverse();

        self.buffer[idx] = b' ';
        idx += 1;

        let _ = self.writer.write(&self.buffer[0..idx]);
        let _ = self.writer.write_fmt(args);
        let _ = self.writer.write(b"\n");
        let _ = self.writer.flush();
    }
}

/// The type returned by [`from_str`] when the string doesn't match any of the log levels.
///
/// [`from_str`]: https://doc.rust-lang.org/std/str/trait.FromStr.html#tymethod.from_str
#[derive(Debug, PartialEq)]
pub struct LevelParseError;

impl fmt::Display for LevelParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("parse log level error")
    }
}

impl error::Error for LevelParseError {}


/// An enum representing the available verbosity levels of the logger.
///
/// Typical usage includes: specifying the `Level` of [`log!`](../../macro.log.html),
/// and comparing a `Level` directly to `Level`
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Level {
    /// The "none" level.
    ///
    /// No logging
    None = 0,
    /// The "emerg" level.
    ///
    /// System is unusable
    Emerg,
    /// The "alert" level.
    ///
    /// Action must be taken immediately
    Alert,
    /// The "crit" level.
    ///
    /// Critical conditions
    Crit,
    /// The "error" level.
    ///
    /// Error conditions
    Error,
    /// The "warn" level.
    ///
    /// Wanrning conditions
    Warn,
    /// The "notice" level.
    ///
    /// Normal but significant condition
    Notice,
    /// The "info" level.
    ///
    /// Information
    Info,
    /// The "debug" level.
    ///
    /// Debug messages
    Debug,
}

static LOG_LEVEL_NAMES: [&str; 9] = [
    "NONE", "EMERG", "ALERT", "CRIT", "ERROR", "WARN", "NOTICE", "INFO", "DEBUG",
];

impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(LOG_LEVEL_NAMES[*self as usize])
    }
}

impl From<usize> for Level {
    fn from(l: usize) -> Self {
        match l {
            0 => Level::None,
            1 => Level::Emerg,
            2 => Level::Alert,
            3 => Level::Crit,
            4 => Level::Error,
            5 => Level::Warn,
            6 => Level::Notice,
            7 => Level::Info,
            8 => Level::Debug,
            _ => panic!("Unknown LogLevel"),
        }
    }
}

impl FromStr for Level {
    type Err = LevelParseError;
    fn from_str(s: &str) -> Result<Level, Self::Err> {
        let opt = LOG_LEVEL_NAMES
            .iter()
            .position(|&name| -> bool {
                if name.len() != s.len() {
                    return false;
                }

                // case insensitive
                s.bytes()
                    .zip(name.bytes())
                    .all(|(a, b)| (a | 0x20) == (b | 0x20))
            })
            .into_iter()
            .map(|idx| Level::from(idx))
            .next();

        match opt {
            Some(o) => Ok(o),
            None => Err(LevelParseError),
        }
    }
}


mod private {
    pub fn localtime(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
        // Copy from https://github.com/tailhook/humantime
        /*
                                         Apache License
                                   Version 2.0, January 2004
                                http://www.apache.org/licenses/

           TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

           1. Definitions.

              "License" shall mean the terms and conditions for use, reproduction,
              and distribution as defined by Sections 1 through 9 of this document.

              "Licensor" shall mean the copyright owner or entity authorized by
              the copyright owner that is granting the License.

              "Legal Entity" shall mean the union of the acting entity and all
              other entities that control, are controlled by, or are under common
              control with that entity. For the purposes of this definition,
              "control" means (i) the power, direct or indirect, to cause the
              direction or management of such entity, whether by contract or
              otherwise, or (ii) ownership of fifty percent (50%) or more of the
              outstanding shares, or (iii) beneficial ownership of such entity.

              "You" (or "Your") shall mean an individual or Legal Entity
              exercising permissions granted by this License.

              "Source" form shall mean the preferred form for making modifications,
              including but not limited to software source code, documentation
              source, and configuration files.

              "Object" form shall mean any form resulting from mechanical
              transformation or translation of a Source form, including but
              not limited to compiled object code, generated documentation,
              and conversions to other media types.

              "Work" shall mean the work of authorship, whether in Source or
              Object form, made available under the License, as indicated by a
              copyright notice that is included in or attached to the work
              (an example is provided in the Appendix below).

              "Derivative Works" shall mean any work, whether in Source or Object
              form, that is based on (or derived from) the Work and for which the
              editorial revisions, annotations, elaborations, or other modifications
              represent, as a whole, an original work of authorship. For the purposes
              of this License, Derivative Works shall not include works that remain
              separable from, or merely link (or bind by name) to the interfaces of,
              the Work and Derivative Works thereof.

              "Contribution" shall mean any work of authorship, including
              the original version of the Work and any modifications or additions
              to that Work or Derivative Works thereof, that is intentionally
              submitted to Licensor for inclusion in the Work by the copyright owner
              or by an individual or Legal Entity authorized to submit on behalf of
              the copyright owner. For the purposes of this definition, "submitted"
              means any form of electronic, verbal, or written communication sent
              to the Licensor or its representatives, including but not limited to
              communication on electronic mailing lists, source code control systems,
              and issue tracking systems that are managed by, or on behalf of, the
              Licensor for the purpose of discussing and improving the Work, but
              excluding communication that is conspicuously marked or otherwise
              designated in writing by the copyright owner as "Not a Contribution."

              "Contributor" shall mean Licensor and any individual or Legal Entity
              on behalf of whom a Contribution has been received by Licensor and
              subsequently incorporated within the Work.

           2. Grant of Copyright License. Subject to the terms and conditions of
              this License, each Contributor hereby grants to You a perpetual,
              worldwide, non-exclusive, no-charge, royalty-free, irrevocable
              copyright license to reproduce, prepare Derivative Works of,
              publicly display, publicly perform, sublicense, and distribute the
              Work and such Derivative Works in Source or Object form.

           3. Grant of Patent License. Subject to the terms and conditions of
              this License, each Contributor hereby grants to You a perpetual,
              worldwide, non-exclusive, no-charge, royalty-free, irrevocable
              (except as stated in this section) patent license to make, have made,
              use, offer to sell, sell, import, and otherwise transfer the Work,
              where such license applies only to those patent claims licensable
              by such Contributor that are necessarily infringed by their
              Contribution(s) alone or by combination of their Contribution(s)
              with the Work to which such Contribution(s) was submitted. If You
              institute patent litigation against any entity (including a
              cross-claim or counterclaim in a lawsuit) alleging that the Work
              or a Contribution incorporated within the Work constitutes direct
              or contributory patent infringement, then any patent licenses
              granted to You under this License for that Work shall terminate
              as of the date such litigation is filed.

           4. Redistribution. You may reproduce and distribute copies of the
              Work or Derivative Works thereof in any medium, with or without
              modifications, and in Source or Object form, provided that You
              meet the following conditions:

              (a) You must give any other recipients of the Work or
                  Derivative Works a copy of this License; and

              (b) You must cause any modified files to carry prominent notices
                  stating that You changed the files; and

              (c) You must retain, in the Source form of any Derivative Works
                  that You distribute, all copyright, patent, trademark, and
                  attribution notices from the Source form of the Work,
                  excluding those notices that do not pertain to any part of
                  the Derivative Works; and

              (d) If the Work includes a "NOTICE" text file as part of its
                  distribution, then any Derivative Works that You distribute must
                  include a readable copy of the attribution notices contained
                  within such NOTICE file, excluding those notices that do not
                  pertain to any part of the Derivative Works, in at least one
                  of the following places: within a NOTICE text file distributed
                  as part of the Derivative Works; within the Source form or
                  documentation, if provided along with the Derivative Works; or,
                  within a display generated by the Derivative Works, if and
                  wherever such third-party notices normally appear. The contents
                  of the NOTICE file are for informational purposes only and
                  do not modify the License. You may add Your own attribution
                  notices within Derivative Works that You distribute, alongside
                  or as an addendum to the NOTICE text from the Work, provided
                  that such additional attribution notices cannot be construed
                  as modifying the License.

              You may add Your own copyright statement to Your modifications and
              may provide additional or different license terms and conditions
              for use, reproduction, or distribution of Your modifications, or
              for any such Derivative Works as a whole, provided Your use,
              reproduction, and distribution of the Work otherwise complies with
              the conditions stated in this License.

           5. Submission of Contributions. Unless You explicitly state otherwise,
              any Contribution intentionally submitted for inclusion in the Work
              by You to the Licensor shall be under the terms and conditions of
              this License, without any additional terms or conditions.
              Notwithstanding the above, nothing herein shall supersede or modify
              the terms of any separate license agreement you may have executed
              with Licensor regarding such Contributions.

           6. Trademarks. This License does not grant permission to use the trade
              names, trademarks, service marks, or product names of the Licensor,
              except as required for reasonable and customary use in describing the
              origin of the Work and reproducing the content of the NOTICE file.

           7. Disclaimer of Warranty. Unless required by applicable law or
              agreed to in writing, Licensor provides the Work (and each
              Contributor provides its Contributions) on an "AS IS" BASIS,
              WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
              implied, including, without limitation, any warranties or conditions
              of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
              PARTICULAR PURPOSE. You are solely responsible for determining the
              appropriateness of using or redistributing the Work and assume any
              risks associated with Your exercise of permissions under this License.

           8. Limitation of Liability. In no event and under no legal theory,
              whether in tort (including negligence), contract, or otherwise,
              unless required by applicable law (such as deliberate and grossly
              negligent acts) or agreed to in writing, shall any Contributor be
              liable to You for damages, including any direct, indirect, special,
              incidental, or consequential damages of any character arising as a
              result of this License or out of the use or inability to use the
              Work (including but not limited to damages for loss of goodwill,
              work stoppage, computer failure or malfunction, or any and all
              other commercial damages or losses), even if such Contributor
              has been advised of the possibility of such damages.

           9. Accepting Warranty or Additional Liability. While redistributing
              the Work or Derivative Works thereof, You may choose to offer,
              and charge a fee for, acceptance of support, warranty, indemnity,
              or other liability obligations and/or rights consistent with this
              License. However, in accepting such obligations, You may act only
              on Your own behalf and on Your sole responsibility, not on behalf
              of any other Contributor, and only if You agree to indemnify,
              defend, and hold each Contributor harmless for any liability
              incurred by, or claims asserted against, such Contributor by reason
              of your accepting any such warranty or additional liability.

           END OF TERMS AND CONDITIONS

           APPENDIX: How to apply the Apache License to your work.

              To apply the Apache License to your work, attach the following
              boilerplate notice, with the fields enclosed by brackets "{}"
              replaced with your own identifying information. (Don't include
              the brackets!)  The text should be enclosed in the appropriate
              comment syntax for the file format. We also recommend that a
              file or class name and description of purpose be included on the
              same "printed page" as the copyright notice for easier
              identification within third-party archives.

           Copyright {yyyy} {name of copyright owner}

           Licensed under the Apache License, Version 2.0 (the "License");
           you may not use this file except in compliance with the License.
           You may obtain a copy of the License at

               http://www.apache.org/licenses/LICENSE-2.0

           Unless required by applicable law or agreed to in writing, software
           distributed under the License is distributed on an "AS IS" BASIS,
           WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
           See the License for the specific language governing permissions and
           limitations under the License.
        */
        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        const DAYS_PER_400Y: i64 = 365*400 + 97;
        const DAYS_PER_100Y: i64 = 365*100 + 24;
        const DAYS_PER_4Y: i64 = 365*4 + 1;

        let days = (secs / 86400) as i64 - LEAPOCH;
        let secs_of_day = secs % 86400;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 { c_cycles -= 1; }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 { q_cycles -= 1; }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 { remyears -= 1; }
        remdays -= remyears * 365;

        let mut year = 2000 +
            remyears + 4*q_cycles + 100*c_cycles + 400*qc_cycles;

        let months = [31,30,31,30,31,31,30,31,30,31,31,29];
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if remdays < *mon_len {
                break;
            }
            remdays -= *mon_len;
        }
        let mday = remdays+1;
        let mon = if mon + 2 > 12 {
            year += 1;
            mon - 10
        } else {
            mon + 2
        };

        let hours = secs_of_day / 3600;
        let minutes = secs_of_day % 3600 / 60;
        let seconds = secs_of_day % 3600 % 60;

        (year as u32, mon as u32, mday as u32, hours as u32, minutes as u32, seconds as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn level_from_usize_panic() {
        let _crash = Level::from(9);
    }

    #[test]
    fn level_from_usize_normal() {
        for i in 0..=8 {
            let lvl = Level::from(i);
            assert_eq!(i, lvl as usize);
        }
    }

    #[test]
    fn level_from_str() {
        assert_eq!(Level::from_str("none"), Ok(Level::None));
        assert_eq!(Level::from_str("NONE"), Ok(Level::None));
        assert_eq!(Level::from_str("none1"), Err(LevelParseError));

        assert_eq!(Level::from_str("emerg"), Ok(Level::Emerg));
        assert_eq!(Level::from_str("EMERG"), Ok(Level::Emerg));
        assert_eq!(Level::from_str("EMERG2"), Err(LevelParseError));

        assert_eq!(Level::from_str("alert"), Ok(Level::Alert));
        assert_eq!(Level::from_str("ALERT"), Ok(Level::Alert));
        assert_eq!(Level::from_str("ALERT3"), Err(LevelParseError));

        assert_eq!(Level::from_str("crit"), Ok(Level::Crit));
        assert_eq!(Level::from_str("CRIT"), Ok(Level::Crit));
        assert_eq!(Level::from_str("crit4"), Err(LevelParseError));

        assert_eq!(Level::from_str("error"), Ok(Level::Error));
        assert_eq!(Level::from_str("Error"), Ok(Level::Error));
        assert_eq!(Level::from_str("5Error5"), Err(LevelParseError));

        assert_eq!(Level::from_str("warn"), Ok(Level::Warn));
        assert_eq!(Level::from_str("WARN"), Ok(Level::Warn));
        assert_eq!(Level::from_str(" warn"), Err(LevelParseError));

        assert_eq!(Level::from_str("notice"), Ok(Level::Notice));
        assert_eq!(Level::from_str("NoTiCe"), Ok(Level::Notice));
        assert_eq!(Level::from_str("motice"), Err(LevelParseError));

        assert_eq!(Level::from_str("info"), Ok(Level::Info));
        assert_eq!(Level::from_str("INFO"), Ok(Level::Info));
        assert_eq!(Level::from_str("iinfoo"), Err(LevelParseError));

        assert_eq!(Level::from_str("debug"), Ok(Level::Debug));
        assert_eq!(Level::from_str("deBUG"), Ok(Level::Debug));
        assert_eq!(Level::from_str("ddd"), Err(LevelParseError));
    }
}
