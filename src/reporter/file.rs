use codespan_reporting::files;
use std::ops::Range;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct File<'a> {
    /// The name of the file.
    name: String,
    /// The source code of the file.
    source: &'a str,
    /// The starting byte indices in the source code.
    line_starts: Vec<usize>,
}

impl<'a> File<'a> {
    pub fn new(name: &'a Path, source: &'a str) -> Self {
        let name = name.to_string_lossy().into_owned();
        let line_starts = files::line_starts(&source).collect();

        Self {
            name,
            source,
            line_starts,
        }
    }

    fn line_start(&self, line_index: usize) -> Result<usize, files::Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(self
                .line_starts
                .get(line_index)
                .expect("failed despite previous check")
                .clone()),
            Ordering::Equal => Ok(self.source.len()),
            Ordering::Greater => Err(files::Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }
}

impl<'a> files::Files<'a> for File<'a> {
    type FileId = ();
    type Name = &'a str;
    type Source = &'a str;

    fn name(&self, _file_id: ()) -> Result<&str, files::Error> {
        Ok(&self.name)
    }

    fn source(&self, _file_id: ()) -> Result<&str, files::Error> {
        Ok(&self.source)
    }

    fn line_index(&self, _file_id: (), byte_index: usize) -> Result<usize, files::Error> {
        self.line_starts
            .binary_search(&byte_index)
            .or_else(|next_line| Ok(next_line - 1))
    }

    fn line_range(&self, _file_id: (), line_index: usize) -> Result<Range<usize>, files::Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}

pub struct Void;

impl<'a> files::Files<'a> for Void {
    type FileId = ();
    type Name = &'a str;
    type Source = &'a str;

    fn name(&self, _file_id: ()) -> Result<&str, files::Error> {
        unreachable!()
    }

    fn source(&self, _file_id: ()) -> Result<&str, files::Error> {
        unreachable!()
    }

    fn line_index(&self, _file_id: (), _byte_index: usize) -> Result<usize, files::Error> {
        unreachable!()
    }

    fn line_range(&self, _file_id: (), _line_index: usize) -> Result<Range<usize>, files::Error> {
        unreachable!()
    }
}
