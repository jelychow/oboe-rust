#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub module: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct DebugLog {
    records: Vec<LogRecord>,
}

impl DebugLog {
    pub fn push(&mut self, module: impl Into<String>, level: LogLevel, message: impl Into<String>) {
        self.records.push(LogRecord {
            module: module.into(),
            level,
            message: message.into(),
        });
    }

    pub fn records(&self) -> &[LogRecord] {
        &self.records
    }
}

#[derive(Debug, Default)]
pub struct Trace {
    enabled: bool,
    active_sections: Vec<String>,
    unsupported_warning_shown: bool,
}

impl Trace {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            active_sections: Vec::new(),
            unsupported_warning_shown: false,
        }
    }

    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn begin_section(&mut self, section_name: impl Into<String>) -> bool {
        if self.enabled {
            self.active_sections.push(section_name.into());
            true
        } else {
            self.unsupported_warning_shown = true;
            false
        }
    }

    pub fn end_section(&mut self) -> bool {
        self.active_sections.pop().is_some()
    }

    pub fn active_sections(&self) -> &[String] {
        &self.active_sections
    }

    pub fn unsupported_warning_shown(&self) -> bool {
        self.unsupported_warning_shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_trace_records_one_warning_state() {
        let mut trace = Trace::disabled();
        assert!(!trace.begin_section("audio"));
        assert!(trace.unsupported_warning_shown());
        assert!(trace.active_sections().is_empty());
    }
}
