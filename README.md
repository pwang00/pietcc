# PietCC

PietCC is a Rust interpreter (and eventually compiler, WIP) for the Piet esoteric language. The interpreter supports automatic codel size inference and various levels of logging based on verbosity. 

## Organization

The repository is organized into four main components:

1. `types`: core types shared between the interpreter, compiler, and parser
2. `interpreter`: core interpreter logic
3. `compiler`: core compiler logic, handles CFG generation and uses Inkwell to generate LLVM IR from CFGs.
4. `main`: main CLI, allows users to run either the interpreter or compiler with a variety of flags.

## Progress

- [x] Interpreter: Functionally complete but needs refactoring and additional beautifying
- [ ] Compiler: In progress (working on code generation and CFG optimization)

## Installation

Clone this repository via

```
git clone https://github.com/pwang00/pietcc
cd pietcc
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

To interpret a program, for example, you can do 

```
cargo run -- images/fizzbuzz.png -i -v 2

Running with codel width of 1 (size of 1)

1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16

```