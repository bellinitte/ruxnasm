# Differences between Uxnasm and Ruxnasm

This file lists all known differences between Uxnasm and Ruxnasm. These features are either already implemented or will be implemented in the future. Please note that the error codes (Exxxx) are not final yet.

#### Validity symbols

- / &mdash; valid in Uxnasm, invalid in Ruxnasm
- \ &mdash; invalid in Uxnasm, valid in Ruxnasm
- \- &mdash; no change in validity or not applicable

## Errors

| N | Uxnasm | Ruxnasm | V[*](#validity-symbols) |
|:-:|--------|---------|:-----------------------:|
| 1 | Ignores any misplaced closing parentheses i.e. the ones that are not a part of any comment because they don't have a matching opening parenthesis. | A misplaced closing parenthesis results in error E0003. | / |
| 2 | A comment doesn't have to be closed at the end of the file. | Any unclosed comments, i.e. opening parentheses that do not have a matching closing parenthesis, result in error E0004. | / |
| 3 | Omitting the sublabel name after a sublabel definition rune is valid and results in a `label/` label, where `label` is a name of the previously defined label. | Omitting the sublabel name after a sublabel definition rune results in error E0006. | / |
| 4 | Allows you to omit a hexadecimal number after the relative pad rune and treats the value as zero. | Reports error E0001 if no hexadecimal number after the relative pad rune is provided. | / |
| 5 | Allows you to provide a string of characters that is not a valid hexadecimal number after the relative pad rune, in which case it ignores the invalid digits. | Reports error E0002 if the string of characters after the relative pad rune is not a valid hexadecimal number (i.e. there are invalid hexadecimal digits in the number string). | / |
| 6 | Omitting the label or a sublabel name in a sublabel path after address runes is valid and specifies a `/sublabel` and `label/` label respectively. | Omitting the label or a sublabel name in a sublabel path after address runes results in error E0008 for labels and error E0009 for sublabels. | / |
| 7 | Label names can have a "`&`" as the first character. This is valid code: <pre>@&label &label .&label</pre> | Label names cannot have a "`&`" as the first character, as it clashes with the `.&label` syntax. Any such label name results in error E0010. | / |
| 8 | Allows you to include "`/`" characters in label and sublabel names. | "`/`" characters in label and sublabel names are invalid, as they make sublabel paths unnecessarily ambiguous, and result in errors E0011 for labels and errors E0012 for sublabels. | / |
| 9 | Omitting the character after a raw character rune is valid and becomes a raw byte with value 0. | Omitting the character after a raw character rune results in error E0021. | / |
| 10 | Ignores all closing brackets. | Still ignores all closing brackets, but any misplaced closing bracket i.e. one that does not have a matching opening bracket results in error E0024. | / |
| 11 | Ignores all opening brackets. | Still ignores all opening brackets, but any opening bracket that does not have a matching closing bracket results in error E0025. | / |
| 12 | Recursive macros result in a segmentation fault when expanded. | Any instance of a direct or undirect recursion in macros is detected at assembly time (except when the recursive macro is never expanded) and reported as error E0026. See [Recursive macros](#recursive-macros) for the details. | - |
| 13 | After a raw character rune, ignores all bytes after the first one. | More than one character or a multibyte Unicode character after a raw character rune results in error E0027. | / |
| 14 | Using the `.&label` syntax without a previously defined label is valid and generates a label out of garbage memory. | Using the `.&label` syntax without a previously defined label results in error E0030. | / |
| 15 | Defining a sublabel without a previously defined label is valid and generates a label out of garbage memory. | Defining a sublabel without a previously defined label results in error E0029. | / |
| 16 | Sublabel paths can have more than one slash. | Sublabel paths with more than one slash are invalid and result in error E0014. | / |
| 17 | Programs smaller than one page (256 bytes) result in an integer underflow in the program length variable during the trimming process, which outputs a 1.2 MB binary full of zeros. For programs equal or larger than 256 bytes, any bytes in the zeroth page are trimmed off. | Any bytes in the zeoth page (first 256 bytes) of the binary result in error E0031. | / |
| 18 | Programs longer that 65536 bytes result in an integer overflow &mdash; the program length wraps back to zero. | Programs longer than 65536 bytes result in error E0028. | / |
| 19 | Specifying a hexadecimal number with more than 4 digits after a pad rune is valid. | Specifying a hexadecimal number with more than 4 digits after a pad rune results in error E0032. | / |

## Quirks

| N | Uxnasm | Ruxnasm | V[*](#validity-symbols) |
|:-:|--------|---------|:-----------------------:|
| 20 | Opening and closing parentheses (i.e. comments) allow you only to enable or disable the parsing. | Comments can be nested. | - |
| 21 | Tokens are split by whitespace. See [Delimiters](#delimiters) for the details and implications. | Splits the tokens not only by whitespace but by the delimiters as well. See [Delimiters](#delimiters) for the details. | - |
| 22 | Opening brace after a macro definition can be omitted. | A macro definition not directly followed by an opening brace is a valid, but empty macro. | - |
| 23 | Label definitions, sublabel definitions, macro definitions, and absolute pads are not allowed in macros. | Definitions and absolute pads are valid in macros. See [Definitions and absolute pads in macros](#definitions-and-absolute-pads-in-macros) for the details. | \ |
| 24 | Comments and brackets are not allowed in macros. | Comments and brackets are valid in macros: <ul><li>Any comment opened in a macro must be closed within that macro, or else the closing brace won't be parsed.</li><li>Bracket matching and nesting behaves exactly as if the macro would be expanded inline, so the brackets can be opened within a macro and closed outside of it or vice versa.</li></ul> | \ |
| 25 | Attempting to define a label that is a valid hexadecimal number or a valid instruction results in "Label name is hex number" and "Label name is invalid" errors, respectively. | Labels can be valid hexadecimal numbers or instructions. Labels must be preceded by an address rune &mdash; they don't clash with numbers or instructions in any way. | \ |
| 26 | Labels and sublabels share the same namespace and the names can have at most 64 characters. Sublabel names are cut off if the sum of the lengths of (1) the scope name, (2) the slash character, and (3) the sublabel name is greater than 64 characters. | Labels and sublabels have separate namespaces and the labels are defined as a pair consisting of the scope name and the sublabel name with 64 characters each. | - |
| 27 | `#x` syntax for specyfing a literal ASCII byte is invalid and results in an "Invalid hexadecimal literal" error. | `#x` syntax produces a valid literal ASCII byte code (LIT byte followed by the ASCII byte based on the character after the `#` rune). | \ |

## Examples

### Delimiters

Uxnasm splits the tokens by whitespace (spaces, tabs, and newlines), which means that for tokens starting with "`(`", "`)`", "`[`", "`]`", "`{`", or "`}`" (tokens in which only the first character matters), any character between the start of the token and a whitespace is ignored. This has some implications regarding comments: the string "`1 (2) 3 ( 4 ) 5 ( 6 )7`" is split into tokens \[`1`, `(2)`, `3`, `(`, `4`, `)`, `5`, `(`, `6`, `)7`\], which are interpreted as \[`1`, `(`, `3`, `(`, `4`, `)`, `5`, `(`, `6`, `)`\], and by taking into account the comment skipping, the final list of tokens is \[`1`, `5`\]. I consider this slightly unintuitive, so Ruxnasm (additionally to whitespaces) separates the tokens by the delimiters: the "`(`", "`)`", "`[`", "`]`", "`{`", and "`}`" characters. In Ruxnasm, the same string "`1 (2) 3 ( 4 ) 5 ( 6 )7`" is tokenized into \[`1`, `3`, `5`, `7`\].

### Definitions and absolute pads in macros

Ruxnasm allows you to put any token in a macro definition, including other macro definitions, label and sublabel definitions, as well as absolute pads. This has the following implications:
- Macro definitions, label definitions, and absolute pads are pretty much useless. They are obviously useless if the macro is never expanded, and also useless when the macro is expanded more than one time &mdash; this causes an error (defining a macro/label with the same name multiple, or a memory overwrite in case of the absolute pad). They are fine if the macro is expanded exactly once, but at this point, you might as well put them outside of the macro.

  Macros can be put inside other macros. For example, this code:
  ```uxntal
  %macro1 { %macro2 { ADD } }
  macro1 macro2
  ```
  results in a single `ADD` instruction.
- Sublabel definitions have an interesting use case:
  ```uxntal
  %define-sublabels { &one $1 &two $1 &three $1 }
  @Label1 define-sublabels @Label2 define-sublabels
  .Label1/two
  ```

### Recursive macros

Uxnasm allows you to put an macro expansion inside of the definition of the same macro, which is the simplest (direct) case of a recursive macro.
```uxntal
%macro { macro }
macro
```
This causes an infinite recursion and a stack overflow during the parsing process. A slightly more intricate example of recursion is the indirect recursion, which involves several macros:
```uxntal
%macro-a { macro-b }
%macro-b { macro-a }
macro-a
```
Ruxnasm detects both cases of recursion during the assembly and reports an appropriate error.
