use std::io::{self, Write};

use indicatif::ProgressBar;
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::APP_NAME;

#[derive(Debug, Clone, Copy)]
pub enum Output {
    Quiet,
    Normal,
    Verbose,
    Progress,
}

impl Output {
    pub fn init_logging(self) -> ProgressBar {
        let level = match self {
            Self::Verbose => Level::DEBUG,
            Self::Normal => Level::INFO,
            Self::Quiet | Self::Progress => Level::WARN,
        };
        let bar = ProgressBar::hidden();
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false)
                    .with_writer(SuspendingStderr(bar.clone())),
            )
            .with(Targets::new().with_target(APP_NAME, level))
            .init();
        bar
    }
}

struct SuspendingStderr(ProgressBar);

impl<'a> MakeWriter<'a> for SuspendingStderr {
    type Writer = SuspendedWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        SuspendedWriter(&self.0)
    }
}

struct SuspendedWriter<'a>(&'a ProgressBar);

impl Write for SuspendedWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.suspend(|| io::stderr().write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.suspend(|| io::stderr().flush())
    }
}
