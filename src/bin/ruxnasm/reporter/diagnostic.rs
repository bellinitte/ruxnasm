use std::ops::Range;

macro_rules! impl_severities {
    ($builder: ident) => {
        pub fn bug() -> $builder {
            $builder {
                severity: Severity::Bug,
                notes: Vec::new(),
            }
        }

        pub fn error() -> $builder {
            $builder {
                severity: Severity::Error,
                notes: Vec::new(),
            }
        }

        pub fn warning() -> $builder {
            $builder {
                severity: Severity::Warning,
                notes: Vec::new(),
            }
        }

        pub fn note() -> $builder {
            $builder {
                severity: Severity::Note,
                notes: Vec::new(),
            }
        }

        pub fn help() -> $builder {
            $builder {
                severity: Severity::Help,
                notes: Vec::new(),
            }
        }
    };
}

pub struct VoidDiagnosticBuilderStage1 {
    severity: Severity,
    notes: Vec<String>,
}

impl VoidDiagnosticBuilderStage1 {
    pub fn with_message(self, message: impl Into<String>) -> VoidDiagnostic {
        VoidDiagnostic {
            severity: self.severity,
            message: message.into(),
            notes: self.notes,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

pub struct VoidDiagnostic {
    severity: Severity,
    message: String,
    notes: Vec<String>,
}

impl<'a> VoidDiagnostic {
    impl_severities!(VoidDiagnosticBuilderStage1);

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

pub struct FileDiagnosticBuilderStage1 {
    severity: Severity,
    notes: Vec<String>,
}

impl FileDiagnosticBuilderStage1 {
    pub fn with_message(self, message: impl Into<String>) -> FileDiagnosticBuilderStage2 {
        FileDiagnosticBuilderStage2 {
            severity: self.severity,
            message: message.into(),
            notes: self.notes,
        }
    }
}

pub struct FileDiagnosticBuilderStage2 {
    severity: Severity,
    message: String,
    notes: Vec<String>,
}

impl FileDiagnosticBuilderStage2 {
    pub fn with_label(self, label: Label) -> FileDiagnostic {
        FileDiagnostic {
            severity: self.severity,
            message: self.message,
            label,
            additional_labels: Vec::new(),
            notes: self.notes,
        }
    }
}

pub struct FileDiagnostic {
    severity: Severity,
    message: String,
    label: Label,
    additional_labels: Vec<Label>,
    notes: Vec<String>,
}

impl<'a> FileDiagnostic {
    impl_severities!(FileDiagnosticBuilderStage1);

    pub fn with_label(mut self, label: Label) -> FileDiagnostic {
        self.additional_labels.push(label);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Severity {
    /// An unexpected bug.
    Bug,
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// A note.
    Note,
    /// A help message.
    Help,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub style: LabelStyle,
    pub span: Range<usize>,
    pub message: String,
}

#[derive(Debug, Copy, Clone)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

impl From<Severity> for codespan_reporting::diagnostic::Severity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Bug => Self::Bug,
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Note => Self::Note,
            Severity::Help => Self::Help,
        }
    }
}

impl From<LabelStyle> for codespan_reporting::diagnostic::LabelStyle {
    fn from(label_style: LabelStyle) -> Self {
        match label_style {
            LabelStyle::Primary => Self::Primary,
            LabelStyle::Secondary => Self::Secondary,
        }
    }
}

impl From<Label> for codespan_reporting::diagnostic::Label<()> {
    fn from(label: Label) -> Self {
        Self {
            style: label.style.into(),
            file_id: (),
            range: label.span,
            message: label.message,
        }
    }
}

impl From<VoidDiagnostic> for codespan_reporting::diagnostic::Diagnostic<()> {
    fn from(diagnostic: VoidDiagnostic) -> Self {
        Self {
            severity: diagnostic.severity.into(),
            code: None,
            message: diagnostic.message,
            labels: Vec::new(),
            notes: diagnostic.notes,
        }
    }
}

impl From<FileDiagnostic> for codespan_reporting::diagnostic::Diagnostic<()> {
    fn from(diagnostic: FileDiagnostic) -> Self {
        let mut labels = vec![diagnostic.label.into()];
        labels.extend(
            diagnostic
                .additional_labels
                .into_iter()
                .map(codespan_reporting::diagnostic::Label::from),
        );
        Self {
            severity: diagnostic.severity.into(),
            code: None,
            message: diagnostic.message,
            labels,
            notes: diagnostic.notes,
        }
    }
}
