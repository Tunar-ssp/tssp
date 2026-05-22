//! Output format and color policy decisions.

/// Machine-readable or human-readable output format.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-oriented output.
    Pretty,
    /// Stable JSON output.
    Json,
    /// Tab-separated output for non-interactive pipelines.
    Tsv,
}

/// User color preference.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorChoice {
    /// Detect terminal capabilities and environment variables.
    Auto,
    /// Always use color unless `NO_COLOR` is set.
    Always,
    /// Never use color.
    Never,
}

/// Deterministic output policy for one command invocation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutputPolicy {
    /// Selected output format.
    pub format: OutputFormat,
    /// Whether ANSI colors should be emitted.
    pub color_enabled: bool,
    /// Whether progress output should be drawn as an interactive bar.
    pub progress_bar_enabled: bool,
}

/// User-requested data output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RequestedOutput {
    /// Human or pipeline output should be selected from terminal state.
    Default,
    /// Stable JSON was requested explicitly.
    Json,
}

/// Whether a stream is connected to an interactive terminal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StreamKind {
    /// Interactive terminal.
    Tty,
    /// Pipe, file, or other non-interactive sink.
    Piped,
}

/// Color-related environment state.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorEnvironment {
    /// No environment variable disables color.
    AllowsColor,
    /// `NO_COLOR` is present.
    NoColor,
}

/// Terminal and flag facts used to choose output behavior.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OutputFacts {
    /// Data output requested by the user.
    pub requested_output: RequestedOutput,
    /// State of stdout.
    pub stdout: StreamKind,
    /// State of stderr.
    pub stderr: StreamKind,
    /// Color-related environment state.
    pub color_environment: ColorEnvironment,
}

impl OutputPolicy {
    /// Builds the output policy from global flags and terminal facts.
    #[must_use]
    pub const fn decide(color_choice: ColorChoice, facts: OutputFacts) -> Self {
        let format = match (facts.requested_output, facts.stdout) {
            (RequestedOutput::Json, _) => OutputFormat::Json,
            (RequestedOutput::Default, StreamKind::Tty) => OutputFormat::Pretty,
            (RequestedOutput::Default, StreamKind::Piped) => OutputFormat::Tsv,
        };

        let color_enabled = match color_choice {
            ColorChoice::Never => false,
            ColorChoice::Always => matches!(facts.color_environment, ColorEnvironment::AllowsColor),
            ColorChoice::Auto => {
                matches!(facts.stdout, StreamKind::Tty)
                    && matches!(facts.color_environment, ColorEnvironment::AllowsColor)
            }
        };

        Self {
            format,
            color_enabled,
            progress_bar_enabled: matches!(facts.stdout, StreamKind::Tty)
                && matches!(facts.stderr, StreamKind::Tty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ColorChoice, ColorEnvironment, OutputFacts, OutputFormat, OutputPolicy, RequestedOutput,
        StreamKind,
    };

    #[test]
    fn json_flag_wins_over_tty_detection() {
        let policy = OutputPolicy::decide(
            ColorChoice::Auto,
            OutputFacts {
                requested_output: RequestedOutput::Json,
                stdout: StreamKind::Tty,
                stderr: StreamKind::Tty,
                color_environment: ColorEnvironment::AllowsColor,
            },
        );

        assert_eq!(policy.format, OutputFormat::Json);
    }

    #[test]
    fn piped_stdout_uses_tsv_without_color() {
        let policy = OutputPolicy::decide(
            ColorChoice::Auto,
            OutputFacts {
                requested_output: RequestedOutput::Default,
                stdout: StreamKind::Piped,
                stderr: StreamKind::Tty,
                color_environment: ColorEnvironment::AllowsColor,
            },
        );

        assert_eq!(policy.format, OutputFormat::Tsv);
        assert!(!policy.color_enabled);
    }

    #[test]
    fn no_color_disables_forced_color() {
        let policy = OutputPolicy::decide(
            ColorChoice::Always,
            OutputFacts {
                requested_output: RequestedOutput::Default,
                stdout: StreamKind::Tty,
                stderr: StreamKind::Tty,
                color_environment: ColorEnvironment::NoColor,
            },
        );

        assert!(!policy.color_enabled);
    }
}
