# Grammar

This document describes the implemented Frag syntax in compact EBNF form.
Semantic constraints such as width checking and valid assignment targets are
documented in [LANGUAGE.md](LANGUAGE.md).

## Lexical Rules

```ebnf
identifier = ("A".."Z" | "a".."z" | "_"),
             { "A".."Z" | "a".."z" | "0".."9" | "_" } ;

number     = decimal | binary | hexadecimal ;
decimal    = digit, { digit | "_" } ;
binary     = "0b", ("0" | "1" | "_"), { "0" | "1" | "_" } ;
hexadecimal = "0x", hex_digit, { hex_digit | "_" } ;

line_comment  = ("//" | "#"), { any_char_except_newline } ;
block_comment = "/*", { any_char }, "*/" ;
```

Keywords:

```text
module input output wire reg const on rising falling if else case bit bool true false
```

## Module

```ebnf
source       = module ;
module       = "module", identifier, "{", module_item, { module_item }, "}" ;
module_item  = declaration | const_declaration | assignment | process ;
```

## Declarations

```ebnf
declaration       = ("input" | "output" | "wire" | "reg"),
                    identifier, ":", type, ";" ;

const_declaration = "const", identifier, ":", type, "=", expression, ";" ;

type              = "bit" | "bool" | unsigned_type ;
unsigned_type     = "u", decimal_width ;
decimal_width     = nonzero_digit, { digit } ;
```

Widths must be in the inclusive range `1..=128`.

## Assignments And Processes

```ebnf
assignment = identifier, "=", expression, ";" ;

process    = "on", edge, "(", identifier, ")",
             "{", assignment, { assignment }, "}" ;

edge       = "rising" | "falling" ;
```

## Expressions

Expressions are listed from lowest to highest precedence.

```ebnf
expression       = logical_or ;
logical_or       = logical_and, { "||", logical_and } ;
logical_and      = bit_or, { "&&", bit_or } ;
bit_or           = bit_xor, { "|", bit_xor } ;
bit_xor          = bit_and, { "^", bit_and } ;
bit_and          = equality, { "&", equality } ;
equality         = comparison, { ("==" | "!="), comparison } ;
comparison       = shift, { ("<" | "<=" | ">" | ">="), shift } ;
shift            = term, { ("<<" | ">>"), term } ;
term             = factor, { ("+" | "-"), factor } ;
factor           = unary, { ("*" | "/" | "%"), unary } ;
unary            = ("!" | "~" | "-"), unary | postfix ;
postfix          = primary, { bit_selection } ;
```

## Primary Expressions

```ebnf
primary = number
        | "true"
        | "false"
        | identifier
        | "(", expression, ")"
        | if_expression
        | case_expression ;

if_expression = "if", expression, "{", expression, "}",
                "else", "{", expression, "}" ;

case_expression = "case", expression, "{",
                  case_arm, { ",", case_arm },
                  [ "," ],
                  "}" ;

case_arm = expression, "=>", expression
         | "else", "=>", expression ;

bit_selection = "[", number, "]"
              | "[", number, ":", number, "]" ;
```

Case expressions require exactly one `else` arm, and it must be last.
Slice ranges must be descending and inside the selected expression width.
