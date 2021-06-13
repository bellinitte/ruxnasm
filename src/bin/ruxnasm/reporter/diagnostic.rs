use std::ops::Range;

macro_rules! impl_severities {
    ($builder: ident) => {
        pub fn bug() -> $builder {
            $builder {
                severity: Severity::Bug,
            }
        }

        pub fn error() -> $builder {
            $builder {
                severity: Severity::Error,
            }
        }

        pub fn warning() -> $builder {
            $builder {
                severity: Severity::Warning,
            }
        }
    };
}

pub struct VoidDiagnosticBuilderStage1 {
    severity: Severity,
}

impl VoidDiagnosticBuilderStage1 {
    pub fn with_message(self, message: impl Into<String>) -> VoidDiagnostic {
        VoidDiagnostic {
            severity: self.severity,
            message: message.into(),
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }
}

pub struct VoidDiagnostic {
    severity: Severity,
    message: String,
    notes: Vec<String>,
    helps: Vec<String>,
}

impl<'a> VoidDiagnostic {
    impl_severities!(VoidDiagnosticBuilderStage1);

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }
}

pub struct FileDiagnosticBuilderStage1 {
    severity: Severity,
}

impl FileDiagnosticBuilderStage1 {
    pub fn with_message(self, message: impl Into<String>) -> FileDiagnosticBuilderStage2 {
        FileDiagnosticBuilderStage2 {
            severity: self.severity,
            message: message.into(),
        }
    }
}

pub struct FileDiagnosticBuilderStage2 {
    severity: Severity,
    message: String,
}

impl FileDiagnosticBuilderStage2 {
    pub fn with_label(self, label: Label) -> FileDiagnostic {
        FileDiagnostic {
            severity: self.severity,
            message: self.message,
            label,
            additional_labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }
}

pub struct FileDiagnostic {
    severity: Severity,
    message: String,
    label: Label,
    additional_labels: Vec<Label>,
    notes: Vec<String>,
    helps: Vec<String>,
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

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Severity {
    Bug,
    Error,
    Warning,
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

impl From<VoidDiagnostic> for Vec<codespan_reporting::diagnostic::Diagnostic<()>> {
    fn from(diagnostic: VoidDiagnostic) -> Self {
        let mut codespan_diagnostics = vec![codespan_reporting::diagnostic::Diagnostic {
            severity: diagnostic.severity.into(),
            code: None,
            message: diagnostic.message,
            labels: Vec::new(),
            notes: Vec::new(),
        }];
        codespan_diagnostics.extend(diagnostic.notes.into_iter().map(|note| {
            codespan_reporting::diagnostic::Diagnostic {
                severity: codespan_reporting::diagnostic::Severity::Note,
                code: None,
                message: note,
                labels: Vec::new(),
                notes: Vec::new(),
            }
        }));
        codespan_diagnostics.extend(diagnostic.helps.into_iter().map(|help| {
            codespan_reporting::diagnostic::Diagnostic {
                severity: codespan_reporting::diagnostic::Severity::Help,
                code: None,
                message: help,
                labels: Vec::new(),
                notes: Vec::new(),
            }
        }));
        codespan_diagnostics
    }
}

impl From<FileDiagnostic> for Vec<codespan_reporting::diagnostic::Diagnostic<()>> {
    fn from(diagnostic: FileDiagnostic) -> Self {
        let mut labels = vec![diagnostic.label.into()];
        labels.extend(
            diagnostic
                .additional_labels
                .into_iter()
                .map(codespan_reporting::diagnostic::Label::from),
        );
        let mut codespan_diagnostics = vec![
            codespan_reporting::diagnostic::Diagnostic {
                severity: diagnostic.severity.into(),
                code: None,
                message: diagnostic.message,
                labels,
                notes: Vec::new(),
            }
        ];
        codespan_diagnostics.extend(diagnostic.notes.into_iter().map(|note| {
            codespan_reporting::diagnostic::Diagnostic {
                severity: codespan_reporting::diagnostic::Severity::Note,
                code: None,
                message: note,
                labels: Vec::new(),
                notes: Vec::new(),
            }
        }));
        codespan_diagnostics.extend(diagnostic.helps.into_iter().map(|help| {
            codespan_reporting::diagnostic::Diagnostic {
                severity: codespan_reporting::diagnostic::Severity::Help,
                code: None,
                message: help,
                labels: Vec::new(),
                notes: Vec::new(),
            }
        }));
        codespan_diagnostics
    }
}
