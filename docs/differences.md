# Differences between Uxnasm and Ruxnasm

This file lists all known differences between Uxnasm and Ruxnasm. These features are either already implemented or will be implemented in the future. Please note that the error codes (Exxxx) are not final yet.

## Errors

N | Uxnasm | Ruxnasm |
--|--------|---------|
1 | Allows you to omit a hexadecimal number after the absolute pad rune and treats the value as zero. | Reports error E0001 if no hexadecimal number after the absolute pad rune is provided. |
2 | Allows you to provide a string of characters that is not a valid hexadecimal number after the absolute pad rune, in which case it ignores the invalid digits. | Reports error E0002 if the string of characters after the absolute pad rune is not a valid hexadecimal number (i.e. there are invalid hexadecimal digits in the number string). |
3 | Ignores any misplaced closing parentheses i.e. the ones that are not a part of any comment because they don't have a matching opening parenthesis. | A misplaced closing parenthesis results in error E0003. |
4 | A comment doesn't have to be closed at the end of the file. | Any unclosed comments, i.e. opening parentheses that do not have a matching closing parenthesis, result in error E0004. |
5 | Omitting the label name after a label definition rune results in a "Label name is a hex number" error. | Omitting the label name after a label definition rune is still an error (E0005), but it specifies that it expected a label name. |
6 | Omitting the sublabel name after a sublabel definition rune is valid and results in a `Label/` label, where `Label` is a name of the previously defined label. | Omitting the sublabel name after a sublabel definition rune results in error E0006. |
7 | Allows you to omit a hexadecimal number after the relative pad rune and treats the value as zero. | Reports error E0001 if no hexadecimal number after the relative pad rune is provided. |
8 | Omitting the label or a sublabel name in a sublabel path after address runes is valid in Uxnasm and specifies a `/sublabel` and `Label/` label respectively. | Omitting the label or a sublabel name in a sublabel path after address runes results in error E0008 for labels and error E0009 for sublabels. |
9 | Label names can have a "`&`" as the first character. This is valid code: <pre>@&label &label .&label</pre> | Label names cannot have a "`&`" as the first character, as it clashes with the `.&label` syntax. Any such label names result in errors E0010. |
10 | Allows you to include "`/`" characters in label and sublabel names. | "`/`" characters in label and sublabel names are invalid, as they make sublabel paths unnecessarily ambiguous, and result in errors E0011 for labels and errors E0012 for sublabels. |
11 | Omitting the macro name after a macro definition rune results in a "Macro name is a hex number" error. | Omitting the macro name after a macro definition rune is still an error (E0013), but it specifies that is expected a macro name. |
12 | Omitting the hexadecimal number after a literal hex rune results in an "Unknown label in second pass" error. | Omitting the hexadecimal number after a literal hex rune is still an error (E0001), but it specifies that it expected a hexadecimal number. |
13 | Specifying a 3-digit hexadecimal number after a literal hex rune results in an "Unknown label in second pass" error. | Specifying a 3-digit hexadecimal number after a literal hex rune is still an error (E0015), but it specifies that the hexadecimal number has an uneven length. |
14 | Specifying a hexadecimal number with more than 4 digits after a literal hex rune results in an "Unknown label in second pass" error. | Specifying a hexadecimal number with more than 4 digits after a literal hex rune is still an error (E0016), but it specifies that the hexadecimal number is too long. |
15 | Omitting the identifier after the address runes results in an "Unknown label in second pass" error. | Omitting the identifier after the address runes is still an error (E0017), but it specifies that it expected an identifier (a label, sublabel, or a sublabel path). |
16 | Omitting the character after a raw character rune is valid and results in a raw byte with value 0. | Omitting the character after a raw character rune results in error E0021. |
17 | Any opening brace outside of a macro definition results in an "Unknown label in first pass" error. | Any opening brace outside of a macro definition still results in error (E0022), but it specifies that it found an opening brace outside of a macro definition. |
18 | A macro definition that is not closed at the end of the file results in a "Macro too large" error. | Any unclosed macro definitions, i.e. opening braces that do not have a matching closing brace, result in error E0023.
19 | Ignores all closing brackets. | Still ignores all closing brackets, but any misplaced closing bracket i.e. one that does not have a matching opening bracket results in error E0024. |
20 | Ignores all opening brackets. | Still ignores all opening brackets, but any opening bracket that does not have a matching closing bracket results in error E0025. |
21 | Recursive macros result in a segmentation fault when invoked. | Any instance of a direct or undirect recursion in macros is detected at assembly time (even when the recursive macro is never invoked) and reported as error E0026. See [Recursive macros](#recursive-macros) for the details. |
22 | After a raw character rune, ignores all bytes after the first one. | More than one character or a multibyte Unicode character after a raw character rune results in error E0027.
23 | Specifying a 1 or 3-digit hexadecimal number after the absolute pad rune is valid. | Specifying a 1 or 3-digit hexadecimal number after the absolute pad rune results in an error E00015 to be more consistent with the literal or raw hexadecimal numbers. |
24 | Specifying a hexadecimal number with more than 4 digits after the absolute pad rune is valid, but the more significant digits are ignored. | Specifying a hexadecimal number with more than 4 digits after the absolute pad rune results in error E0016. |
25 | Defining a sublabel without a previously defined label is valid and generates a label out of garbage memory. | Defining a sublabel without a previously defined label results in error E0030.
26 | Using the `.&label` syntax without a previously defined label is valid and generates a label out of garbage memory. | Using the `.&label` syntax without a previously defined label results in error E0030.

## Quirks

N | Uxnasm | Ruxnasm |
--|--------|---------|
27 | Opening and closing parentheses (i.e. comments) allow you only to enable or disable the parsing. | Comments can be nested. |
28 | Tokens are split by whitespace. See [Delimiters](#delimiters) for the details and implications. | Splits the tokens not only by whitespace but by the delimiters as well. See [Delimiters](#delimiters) for the details. |
29 | Opening brace after a macro definition can be omitted. | A macro definition not followed by an opening brace is a valid, but empty macro. |
30 | Label definitions, sublabel definitions, macro definitions, and absolute pads are not allowed in macros. | Definitions and absolute pads are also valid in macros. See [Definitions and absolute pads in macros](#definitions-and-absolute-pads-in-macros) for the details. |
31 | During a macro definition, strings that represent the tokens are copied to the contents of a macro and are parsed only during the macro invocation. This means that if the macro is never used, it can contain invalid tokens. | Tokens are parsed during macro definitions. |

## Examples

### Delimiters

Uxnasm split the tokens by whitespace (spaces, tabs, and newlines), which means that for tokens starting with "`(`", "`)`", "`[`", "`]`", "`{`", or "`}`" (tokens in which only the first character matters), any characters between the start of the token and a whitespace are ignored. This has some implications regarding comments: the string "`1 (2) 3 ( 4 ) 5 ( 6 )7`" is split into tokens \[`1`, `(2)`, `3`, `(`, `4`, `)`, `5`, `(`, `6`, `)7`\], which are interpreter as \[`1`, `(`, `3`, `(`, `4`, `)`, `5`, `(`, `6`, `)`\], and by taking into account the comments, the final list of tokens is \[`1`, `5`\]. I consider this slightly unintuitive, so Ruxnasm, additionally to whitespaces, separates the tokens also by the delimiters: the "`(`", "`)`", "`[`", "`]`", "`{`", and "`}`" characters. In Ruxnasm, the same string "`1 (2) 3 ( 4 ) 5 ( 6 )7`" is tokenized into \[`1`, `3`, `5`, `7`\].

### Definitions and absolute pads in macros

Ruxnasm allows you to put any token in a macro definition, including other macro definitions, label and sublabel definitions, as well as absolute pads. This has the following implications:
- macro definitions, label definitions, and absolute pads are pretty much useless. They are obviously useless if the macro is never invoked, and also useless when the macro is invoked more than one time &mdash; this causes an error (defining a macro/label with the same name multiple, or a memory overwrite in case of the absolute pad). They are fine if the macro is invoked exactly once, but at this point, you might as well put them outside of the macro.

  Macros can be put inside other macros. For example, this code:
  ```tal
  %macro1 { %macro2 { ADD } }
  macro1 macro2
  ```
  results in a single `ADD` instruction.
- sublabel definitions have an interesting use case:
  ```tal
  %define-sublabels { &one $1 &two $1 &three $1 }
  @Label1 define-sublabels @Label2 define-sublabels
  .Label1/two
  ```

### Recursive macros

Uxnasm allows you to put an invocation of a macro inside of the definition of the same macro, which is the simplest (direct) case of a recursive macro.
```tal
%macro { macro }
macro
```
This causes an infinite recursion and a stack overflow during the parsing process. A slightly more intricate example of recursion is the indirect recursion, which involves several macros:
```tal
%macro-a { macro-b }
%macro-b { macro-a }
macro-a
```
Ruxnasm detects both cases of recursion during the assembly and reports an appropriate error.