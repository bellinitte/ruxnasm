pub enum Error {
    NoInputProvided,
    NoOutputProvided,
    UnexpectedArgument { argument: String },
    UnrecognizedOption { option: String },
}
