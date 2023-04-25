# PietCC

PietCC is a Rust interpreter and compiler for the Piet esoteric language.

## Organization

The repository is organized into four main components:

1. `types`: core types shared between the interpreter, compiler, and parser
2. `interpreter`: core interpreter logic
3. `compiler`: core compiler logic, handles CFG generation and uses Inkwell to generate LLVM IR from CFGs.
4. `parser`: core image parsing logic, handles image loading and pixel/codel operations.
4. `main`: main CLI, allows users to run either the interpreter or compiler with a variety of flags.

## Dependencies
- LLVM libraries (14.0.0), including clang and llc.

## Progress

- [x] Interpreter: functionally complete for all images with correct pixel colors.  
- [ ] Compiler: in active development; partially functional. White codel tracing is not currently supported even though pietcc can correctly compile some images with white codels.  Additionally, some instructions are probably buggy, so will go and fix those.

## TODO

Interpreter: 

* Add functionality to allow treating unknown colors as white / black 

Compiler:

* Add white block tracing support.  This functionality already exists in the interpreter, so porting it over to the compiler shouldn't take too long.  
* Fix bugs in Piet instructions.  I don't know which instructions are buggy exactly, but some compiled programs exhibit different behavior than when interpreted, so I know for a fact that they exist.

## Installation

Clone this repository via

```
git clone https://github.com/pwang00/pietcc
cd pietcc
```

## Building PietCC

To build PietCC, run

```bash
cargo build --release
mv target/release/pietcc .
```

Alternatively, it is possible to combine the build / run workflow via 

`cargo run --release <image> --flags`, but this attempts to check for changes to the PietCC source on every run, so is not recommended.

## Interpreting Piet programs

PietCC provides several options for interpreting programs, as shown below.

```
USAGE:
    pietcc [OPTIONS] <input>

ARGS:
    <input>    Piet source file to interpret

OPTIONS:
...
    -i, --interpret                Interpret the given program
    -o, --output <out>             Output an executable into <file> [default: program.out]
    -s, --size <codel_size>        Interpret or compile with a supplied codel size
    -v, --verbosity <verbosity>    Sets the interpreter's verbosity

```

PietCC will by default try to infer the codel width of the program.  The heuristic used computes the gcd of all the block widths and heights with each other and the program width / height, and will produce a correct estimate of the codel width with high probability.  However, to correctly interpret some programs, supplying the size flag with a corresponding value for the codel width is necessary.

The `images/` directory contains a list of sample programs.  

Here's an example run with [fizzbuzz.png](https://github.com/pwang00/pietcc/blob/main/images/fizzbuzz.png)`:

<img src="https://github.com/pwang00/pietcc/blob/main/images/fizzbuzz.png" alt="Piet FizzBuzz" width="512"/>

```
./pietcc images/fizzbuzz.png -i -v 2

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

Doing `cargo run --release images/fizzbuzz.png -i -v 2` will also work.

## Compiling Piet Programs

PietCC supports emitting executables, LLVM IR, and LLVM bitcode.  The latter two options can be useful for targeting other architectures other than x86_64.  The relevant flags are shown below.

```
USAGE:
    pietcc [OPTIONS] <input>

ARGS:
    <input>    Piet source file to interpret

OPTIONS:
    -d, --default <use_default>    Interpret or compile with a codel size of 1
        --emit-llvm                Emit LLVM IR for a given Piet program
        --emit-llvm-bitcode        Emit LLVM bitcode for a given Piet program
    -i, --interpret                Interpret the given program
    -o, --output <out>             Output an executable into <file> [default: program.out]
    -s, --size <codel_size>        Interpret or compile with a supplied codel size
```

To compile a Piet program to an ELF executable, LLVM IR, and LLVM bitcode respectively, do

* `./pietcc <image> -o <output>`
* `./pietcc <image> -o <output> --emit-llvm`
* `./pietcc <image> -o <output> --emit-llvm-bitcode`


Here are some example images with compilation logs:

[piet_pi.png](https://github.com/pwang00/pietcc/blob/main/images/piet_pi.png)

<img src="https://github.com/pwang00/pietcc/blob/main/images/piet_pi_big.png" alt="Piet Pi"/>

```
$ ./pietcc images/piet_pi.png -o piet_pi
$ ./piet_pi 
31405
Stack (0 values): 1052674 
```

[hw1-11.gif](https://github.com/pwang00/pietcc/blob/main/images/hw1-11.gif)

<img src="https://github.com/pwang00/pietcc/blob/main/images/hw1-11.gif" alt="Hello World with Codel Size 11"/>

```
$ ./pietcc images/hw1-11.gif -o hw1-11
$ ./hw1-11 
Hello, world!
Stack (0 values): 1052674 
```

Note that codel size inference is being done implicitly.

## Generating control flow graph from Piet LLVM IR

Visualizing a CFG from LLVM IR can be helpful.  As an example, here's a generated CFG for a program that simply push, pops, and dups.

[test2_upscaled.png](https://github.com/pwang00/pietcc/blob/main/images/test2.png)

<img src="https://github.com/pwang00/pietcc/blob/main/images/test2.png" alt="A sample program that does nothing"/>