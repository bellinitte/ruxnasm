use diagnostic::{FileDiagnostic, VoidDiagnostic};
use file::{File, Void};
use std::{path::Path, sync::RwLock};

mod diagnostic;
mod display;
mod file;

pub struct VoidReporter {
    pub writer: RwLock<codespan_reporting::term::termcolor::StandardStream>,
    pub config: codespan_reporting::term::Config,
}

impl VoidReporter {
    pub fn new() -> Self {
        Self {
            writer: RwLock::new(codespan_reporting::term::termcolor::StandardStream::stderr(
                codespan_reporting::term::termcolor::ColorChoice::Always,
            )),
            config: codespan_reporting::term::Config {
                display_style: codespan_reporting::term::DisplayStyle::Rich,
                tab_width: 2,
                #[cfg(windows)]
                styles: with_blue(codespan_reporting::term::termcolor::Color::Cyan),
                #[cfg(not(windows))]
                styles: with_blue(codespan_reporting::term::termcolor::Color::Blue),
                chars: codespan_reporting::term::Chars::box_drawing(),
                start_context_lines: 3,
                end_context_lines: 1,
            },
        }
    }

    pub fn promote<'a>(self, file_path: &'a Path, file_contents: &'a str) -> FileReporter<'a> {
        FileReporter {
            file: File::new(file_path, file_contents),
            writer: self.writer,
            config: self.config,
        }
    }

    pub fn emit(&self, diagnostic: VoidDiagnostic) {
        let codespan_diagnostics: Vec<codespan_reporting::diagnostic::Diagnostic<()>> = diagnostic.into();
        for codespan_diagnostic in codespan_diagnostics {
            let _ = codespan_reporting::term::emit(
                &mut self.writer.write().unwrap().lock(),
                &self.config,
                &Void,
                &codespan_diagnostic,
            );
        }
    }
}

pub struct FileReporter<'a> {
    pub file: File<'a>,
    pub writer: RwLock<codespan_reporting::term::termcolor::StandardStream>,
    pub config: codespan_reporting::term::Config,
}

impl<'a> FileReporter<'a> {
    pub fn demote(self) -> VoidReporter {
        VoidReporter {
            writer: self.writer,
            config: self.config,
        }
    }

    pub fn emit(&self, diagnostic: FileDiagnostic) {
        let codespan_diagnostics: Vec<codespan_reporting::diagnostic::Diagnostic<()>> = diagnostic.into();
        for codespan_diagnostic in codespan_diagnostics {
            let _ = codespan_reporting::term::emit(
                &mut self.writer.write().unwrap().lock(),
                &self.config,
                &self.file,
                &codespan_diagnostic,
            );
        }
    }
}

fn with_blue(blue: codespan_reporting::term::termcolor::Color) -> codespan_reporting::term::Styles {
    use codespan_reporting::term::{
        termcolor::{Color, ColorSpec},
        Styles,
    };

    let mut header = ColorSpec::new().set_bold(true).set_intense(true).clone();

    Styles {
        header_bug: header.clone().set_fg(Some(Color::Red)).clone(),
        header_error: header.clone().set_fg(Some(Color::Red)).clone(),
        header_warning: header.clone().set_fg(Some(Color::Yellow)).clone(),
        header_note: header.clone().set_fg(Some(Color::Green)).clone(),
        header_help: header.clone().set_fg(Some(Color::Cyan)).clone(),
        header_message: header.clone(),

        primary_label_bug: header.clone().set_fg(Some(Color::Red)).clone(),
        primary_label_error: header.clone().set_fg(Some(Color::Red)).clone(),
        primary_label_warning: header.clone().set_fg(Some(Color::Yellow)).clone(),
        primary_label_note: header.clone().set_fg(Some(Color::Green)).clone(),
        primary_label_help: header.clone().set_fg(Some(Color::Cyan)).clone(),
        secondary_label: header.clone().set_fg(Some(blue)).clone(),

        line_number: header.clone().set_fg(Some(blue)).clone(),
        source_border: header.clone().set_fg(Some(blue)).clone(),
        note_bullet: header.set_fg(Some(blue)).clone(),
    }
}
