# Piet Compiler and Interpreter

PietCC is a Rust interpreter (and eventually compiler, WIP) for the Piet esoteric language. The interpreter supports four levels of logging.

## Organization

The repository is organized into four main components:

1. `types`: Core types shared between the interpreter, compiler, and parser
2. `interpreter`: Interprets Piet programs and supports automatic codel size inference
3. `compiler`: Generates control flow graphs for Piet programs and uses Inkwell to generate LLVM IR
4. `main`: Parses command line arguments and runs either the compiler or interpreter depending on the user's choice

## Progress

- [x] Interpreter: Functionally complete but needs refactoring and additional beautifying
- [ ] Compiler: In progress (working on code generation and CFG optimization)

## Installation

Clone this repository via

```
git clone https://github.com/yourusername/piet-compiler-interpreter.git
```

## Usage

```
pietcc 
Piet compiler and interpreter

USAGE:
    pietcc [OPTIONS] <input>

ARGS:
    <input>    Piet source file to interpret

OPTIONS:
    -d, --default <use_default>    Interpret or compile with a codel size of 1
    -h, --help                     Print help information
    -i, --interpret                Interpret the given program
    -o, --output <out>             Place the output into <file>
    -s, --size <codel_size>        Interpret or compile with a supplied codel size
    -v, --verbosity <verbosity>    Sets the interpreter's verbosity
```

To interpret