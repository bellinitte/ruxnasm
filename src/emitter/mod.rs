use super::{Anomalies, Span, Spanned, Token};
pub use anomalies::Warning;
use std::collections::HashMap;

mod anomalies;

struct Label {
    name: String,
    address: u16,
    references: usize,
}

pub(super) fn emit(tokens: &[Spanned<Token>]) -> (Vec<u8>, Anomalies) {
    let mut binary = Vec::new();
    let mut anomalies = Anomalies::new();

    (binary, anomalies)
}

fn pass_1(tokens: &[Spanned<Token>]) {
    let mut address: u16 = 0;
    let mut label: Option<&String> = None;

    let mut macros: HashMap<String, (Vec<Spanned<Token>>, Span)> = HashMap::new();
    let mut labels: HashMap<String, (u16, Span)> = HashMap::new();
    let mut sublabels: HashMap<(Option<String>, String), (u16, Span)> = HashMap::new();

    let mut anomalies = Anomalies::new();

    let mut tokens = tokens.into_iter();

    loop {
        match tokens.next() {
            Some(Spanned {
                node: Token::PadAbsolute(pad),
                ..
            }) => address = *pad as u16,
            Some(Spanned {
                node: Token::MacroDefine(name),
                span: macro_define_span,
            }) => {
                // TODO: Check for duplicates
                // TODO: Check for length
                let mut items: Vec<Spanned<Token>> = Vec::new();
                let mut macro_span = *macro_define_span;
                'macro_items: loop {
                    match tokens.next() {
                        Some(Spanned {
                            node: Token::ClosingBrace,
                            span,
                        }) => {
                            macro_span = Span::combine(&macro_span, &span);
                            break 'macro_items;
                        }
                        Some(spanned) => {
                            items.push(spanned.to_owned());
                            macro_span = Span::combine(&macro_span, &spanned.span);
                        }
                        None => {
                            break 'macro_items;
                        }
                    }
                }
                macros.insert(name.clone(), (items, macro_span));
            }
            Some(Spanned {
                node: Token::ClosingBrace,
                span,
            }) => {
                anomalies.push_warning(Warning::ClosingBraceMisplaced { span: *span });
            }
            Some(Spanned {
                node: Token::LabelDefine(name),
                span,
            }) => {
                // TODO: Check for duplicates
                labels.insert(name.clone(), (address, *span));
                label = Some(name);
            }
            Some(Spanned {
                node: Token::SublabelDefine(name),
                span,
            }) => {
                sublabels.insert((label.cloned(), name.clone()), (address, *span));
            }
            Some(Spanned {
                node: Token::RawHexByte(_),
                ..
            }) => address += 1,
            Some(Spanned {
                node: Token::RawHexShort(_),
                ..
            }) => address += 2,
            Some(Spanned {
                node: Token::Instruction(_),
                ..
            }) => address += 1,
            Some(Spanned {
                node: Token::RawChar(_),
                ..
            }) => address += 1,
            Some(Spanned {
                node: Token::LiteralZeroPageAddress(_),
                ..
            }) => address += 2,
            Some(Spanned {
                node: Token::LiteralRelativeAddress(_),
                ..
            }) => address += 2,
            Some(Spanned {
                node: Token::RawAddress(_),
                ..
            }) => address += 2,
            Some(Spanned {
                node: Token::LiteralAbsoluteAddress(_),
                ..
            }) => address += 3,
            Some(Spanned {
                node: Token::PadRelative(pad),
                ..
            }) => address += *pad as u16,
            Some(Spanned {
                node: Token::LiteralHexByte(_),
                ..
            }) => address += 2,
            Some(Spanned {
                node: Token::LiteralHexShort(_),
                ..
            }) => address += 3,
            Some(Spanned {
                node: Token::RawWord(word),
                ..
            }) => address += word.len() as u16,
            Some(Spanned {
                node: Token::MacroInvoke(name),
                ..
            }) => {}
            None => todo!(),
        }
    }
}
