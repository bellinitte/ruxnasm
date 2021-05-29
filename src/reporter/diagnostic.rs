use ruxnasm::Span;

pub struct VoidDiagnosticBuilderStage1;

impl VoidDiagnosticBuilderStage1 {
    pub fn with_message(self, message: impl Into<String>) -> VoidDiagnostic {
        VoidDiagnostic {
            message: message.into(),
        }
    }
}

pub struct VoidDiagnostic {
    message: String,
}

impl<'a> VoidDiagnostic {
    pub fn error() -> VoidDiagnosticBuilderStage1 {
        VoidDiagnosticBuilderStage1
    }

    pub fn message(&'a self) -> &'a str {
        &self.message
    }
}

pub struct FileDiagnosticBuilderStage1;

impl FileDiagnosticBuilderStage1 {
    pub fn with_message(self, message: impl Into<String>) -> FileDiagnosticBuilderStage2 {
        FileDiagnosticBuilderStage2 {
            message: message.into(),
        }
    }
}

pub struct FileDiagnosticBuilderStage2 {
    message: String,
}

impl FileDiagnosticBuilderStage2 {
    pub fn with_label(self, label: Label) -> FileDiagnostic {
        FileDiagnostic {
            message: self.message,
            label,
            additional_labels: Vec::new(),
        }
    }
}

pub struct FileDiagnostic {
    message: String,
    label: Label,
    additional_labels: Vec<Label>,
}

impl<'a> FileDiagnostic {
    pub fn error() -> FileDiagnosticBuilderStage1 {
        FileDiagnosticBuilderStage1
    }

    pub fn with_label(mut self, label: Label) -> FileDiagnostic {
        self.additional_labels.push(label);
        self
    }

    pub fn message(&'a self) -> &'a str {
        &self.message
    }

    pub fn labels(&self) -> Labels {
        Labels {
            label: self.label.clone(),
            additional_labels: self.additional_labels.clone(),
            counter: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub style: LabelStyle,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Copy, Clone)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

pub struct Labels {
    label: Label,
    additional_labels: Vec<Label>,
    counter: usize,
}

impl Iterator for Labels {
    type Item = Label;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.counter == 0 {
            Some(self.label.clone())
        } else {
            self.additional_labels.get(self.counter - 1).cloned()
        };
        self.counter += 1;
        result
    }
}
