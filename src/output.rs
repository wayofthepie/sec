use crate::input::HandlerResult;
use std::io::Write;

pub fn write_result<W: Write>(
    result: HandlerResult,
    mut output: TerminalOutput<W>,
) -> anyhow::Result<()> {
    match result {
        HandlerResult::Insert(_) => output.write("Secret saved."),
        HandlerResult::Retrieve(value) => output.write(value.as_ref()),
    }
}

pub struct TerminalOutput<W> {
    writer: W,
}

impl<W: Write> TerminalOutput<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write(&mut self, message: &str) -> anyhow::Result<()> {
        Ok(self.writer.write_all(message.as_bytes())?)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        input::HandlerResult,
        output::{write_result, TerminalOutput},
        secrets::ZeroizedString,
    };
    use memfile::{CreateOptions, MemFile};

    #[test]
    fn result_of_insert_should_write_success_if_secret_saved() {
        let mut buf = Vec::new();
        let output = TerminalOutput::new(&mut buf);
        let file = MemFile::create("test", CreateOptions::default())
            .unwrap()
            .into_file();
        let result = HandlerResult::Insert(file);
        write_result(result, output).unwrap();
        let message = std::str::from_utf8(&buf).unwrap();
        assert_eq!(message, "Secret saved.");
    }

    #[test]
    fn result_of_retrieve_should_write_value() {
        let value = ZeroizedString::new("value".to_owned());
        let mut buf = Vec::new();
        let output = TerminalOutput::new(&mut buf);
        let result = HandlerResult::Retrieve(value.clone());
        write_result(result, output).unwrap();
        let message = std::str::from_utf8(&buf).unwrap();
        assert_eq!(message, &*value);
    }
}
