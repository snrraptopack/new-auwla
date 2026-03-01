use ariadne::{Color, Label, Report, ReportKind, Source};
use auwla_ast::Span;

pub enum Level {
    Error,
    Warning,
    Note,
}

pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub labels: Vec<(Span, String)>,
    pub help: Option<String>,
    pub file_path: String,
}

impl Diagnostic {
    pub fn new(level: Level, message: impl Into<String>, file_path: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            labels: Vec::new(),
            help: None,
            file_path: file_path.into(),
        }
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push((span, message.into()));
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn emit(&self, source: &str) {
        let (kind, color) = match self.level {
            Level::Error => (ReportKind::Error, Color::Red),
            Level::Warning => (ReportKind::Warning, Color::Yellow),
            Level::Note => (ReportKind::Advice, Color::Cyan),
        };

        let offset = self.labels.first().map(|(s, _)| s.start).unwrap_or(0);

        let mut builder =
            Report::<(String, Span)>::build(kind, (self.file_path.clone(), offset..offset))
                .with_message(&self.message);

        for (span, label_msg) in &self.labels {
            builder = builder.with_label(
                Label::new((self.file_path.clone(), span.clone()))
                    .with_message(label_msg)
                    .with_color(color),
            );
        }

        if let Some(help) = &self.help {
            builder = builder.with_help(help);
        }

        builder
            .finish()
            .eprint((self.file_path.clone(), Source::from(source)))
            .unwrap();
    }
}
