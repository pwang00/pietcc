# PietCC

PietCC is a Rust interpreter and compiler for the [Piet](https://www.dangermouse.net/esoteric/piet.html) esoteric language using [inkwell](https://github.com/TheDan64/inkwell/tree/master) and LLVM as a backend / IR generator.  To read more about the compiler, visit this [writeup](https://github.com/pwang00/pietcc/blob/main/Compiler.md). 

## Organization

1. [piet_core](https://github.com/pwang00/pietcc/tree/main/types): core Piet language types
2. [interpreter](https://github.com/pwang00/pietcc/tree/main/interpreter): core interpreter logic
3. [compiler](https://github.com/pwang00/pietcc/tree/main/compiler): core compiler logic, handles CFG generation and uses Inkwell to generate LLVM IR from CFGs
4. [parser](https://github.com/pwang00/pietcc/tree/main/parser): core image parsing logic, handles image loading and pixel/codel operations
4. [src](https://github.com/pwang00/pietcc/tree/main/src): main CLI, allows users to run either the interpreter or compiler with a variety of flags

## Dependencies

- Rust 1.89+ (Stable, Beta, or Nightly), for inkwell
- LLVM libraries, including clang and llc, for generating IR / lowering to assembly.

## Progress

### Interpreter

- Supports treating unknown colors as white / black.

### Compiler 

- Supports white block tracing and elimination
- Supports nontermination detection for certain classes of programs
- Supports running LLVM module optimization passes via inkwell
- Supports static evaluation / constant folding


## TODO

### Compiler

* Develop more optimizations and respective optimization passes at the CFG level

## Installation

Clone this repository via

```
git clone https://github.com/pwang00/pietcc
cd pietcc
```

## Building PietCC

To build PietCC, first make sure you have supported versions of rustc and LLVM on your system.  You may need to modify inkwell's cargo feature flags in the Cargo.tomls of the [project root](https://github.com/pwang00/pietcc/blob/main/Cargo.toml) and [compiler](https://github.com/pwang00/pietcc/blob/main/compiler/Cargo.toml).  This project sets them at LLVM 18.1:

```
inkwell = { version = "0.7.0", features = ["llvm18-1"] }
```

However, other versions are supported as well per the inkwell docs:

| LLVM Version | Cargo Feature Flag |
| :----------: | :-----------: |
| 4.0.x        | llvm4-0       |
| 5.0.x        | llvm5-0       |
| 6.0.x        | llvm6-0       |
| 7.0.x        | llvm7-0       |
| 8.0.x        | llvm8-0       |
| 9.0.x        | llvm9-0       |
| 10.0.x       | llvm10-0      |
| 11.0.x       | llvm11-0      |
| 12.0.x       | llvm12-0      |
| 13.0.x       | llvm13-0      |
| 14.0.x       | llvm14-0      |
| 15.0.x       | llvm15-0      |
| 16.0.x       | llvm16-0      |
| 17.0.x       | llvm17-0      |
| 18.1.x       | llvm18-1      |

so replace `features = ["llvm18-1"]` with the desired cargo feature flag.

Once you have verified that the LLVM version is correct, run

```bash
cargo build --release
mv target/release/pietcc .
```

Alternatively, it is possible to combine the build / run workflow via 

`cargo run --release <image> <flags>`, but this attempts to check for changes to the PietCC source on every run, so is not recommended.

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
    -s, --size <codel_size>        Interpret or compile with a supplied codel size (must divide
                                   program height and width)
    -v, --verbosity <verbosity>    Sets the interpreter's verbosity
    --ub                           Treats unknown pixels as black (default: error)
    --uw                           Treats unknown pixels as white (default: error)
```

PietCC will by default try to infer the codel width of the program.  The heuristic used computes the gcd of all the block widths and heights with each other and the program width / height, and will produce a correct estimate of the codel width with high probability.  However, to correctly interpret some programs, supplying the size flag with a corresponding value for the codel width is necessary.

The `images/` directory contains a list of sample programs.  

Here's an example run with [fizzbuzz.png](https://github.com/pwang00/pietcc/blob/main/images/fizzbuzz.png):

<img src="https://github.com/pwang00/pietcc/blob/main/images/fizzbuzz.png" alt="Piet FizzBuzz" width="256"/>

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

## Compiling Piet programs

PietCC supports emitting executables, LLVM IR, and LLVM bitcode.  The latter two options can be useful for targeting other architectures other than x86_64. The relevant flags are shown below.

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
        --o1                       Sets the compiler optimization level to 1 (LLVM Less)
        --o2                       Sets the compiler optimization level to 2 (LLVM Default)
        --o3                       Sets the compiler optimization level to 3 (LLVM Aggressive)
    -s, --size <codel_size>        Interpret or compile with a supplied codel size (must divide
                                   program height and width)
    --ub                           Treats unknown pixels as black (default: error)
    --uw                           Treats unknown pixels as white (default: error)
    -w, --warn-nt                  Attempts to detect nontermination behavior in a Piet program during compilation
```

To compile a Piet program to an ELF executable, LLVM IR, and LLVM bitcode respectively, do

* `./pietcc <image> -o <output>`
* `./pietcc <image> -o <output> --emit-llvm`
* `./pietcc <image> -o <output> --emit-llvm-bitcode`

To specify an optimization level while compiling, do

`./pietcc <image> --o[1|2|3] -o <output>`

To specify behavior when encountering unknown pixels, do

`./pietcc <image> --[ub|uw] -o <output>`

and to specify warning about nontermination, do

`./pietcc <image> -w -o <output>`

### Terminating Piet programs

Here are some example terminating Piet program images with compilation logs:

[Piet Pi Approximation](https://github.com/pwang00/pietcc/blob/main/images/piet_pi.png)

<img src="https://github.com/pwang00/pietcc/blob/main/images/piet_pi_big.png" alt="Piet Pi"/>

```
$ ./pietcc images/piet_pi.png -o piet_pi
$ ./piet_pi 
31405


Stack empty
```

[Hello World 1](https://github.com/pwang00/pietcc/blob/main/images/hw1-11.gif)

<img src="https://github.com/pwang00/pietcc/blob/main/images/hw1-11.gif" alt="Hello World with Codel Size 11"/>

```
$ ./pietcc images/hw1-11.gif -o hw1-11
$ ./hw1-11 
Hello, world!
Stack empty
```

[Piet Brainfuck Interpreter](https://github.com/pwang00/pietcc/blob/main/images/piet_bfi.gif)

<img src="https://github.com/pwang00/pietcc/blob/main/images/piet_bfi.gif" alt="Piet Brainfuck interpreter" width="300"/>

Per the relevant section under [piet samples](https://www.dangermouse.net/esoteric/piet/samples.html)

"The Piet program takes a Brainfuck program, and its input (seperated by |), from STDIN and prints the output of the Brainfuck program to STDOUT. E.g. the input ",+>,+>,+>,+.<.<.<.|sdhO" would generate the output 'Piet'"

```
$ ./pietcc images/piet_bfi.gif -o piet_bfi
$ ./piet_bfi 
Enter char: ,+>,+>,+>,+.<.<.<.|sdhO
Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Enter char: Piet
Stack (size 29): 18 0 7 18 80 0 105 0 101 0 116 44 43 62 44 43 62 44 43 62 44 43 46 60 46 60 46 60 46 
```

[Piet text-based quest](https://github.com/pwang00/pietcc/blob/main/images/pietquest.png)

<img src="https://github.com/pwang00/pietcc/blob/main/images/pietquest.png" alt="Piet text-based quest"/>

(This one takes a really long time to compile--upon profiling, I think it's because llc takes a long time to verify the input LLVM IR module.)

```
$ ./pietcc images/pietquest.png -o pietquest
$ ./pietquest
================
= Piet's Quest =
================


You find yourself in a rather dark studio.
There is an easel here.
There is a ladder leading down.

Please select:

1 - paint
2 - go down ladder
Enter char: 2
Enter char: 
You find yourself in a dusty, dim hallway.
There is a door to the kitchen here.
There is a door to the bedroom here.
There is a rickety loft ladder here.

Where do you want to go today?
1 - kitchen
2 - bedroom
3 - loft
Enter char: 1
Enter char: 

You find yourself in a well-stocked kitchen.
It smells invitingly of apple pancake.
Your wife is here.
She gives you a look.
```

Note that in all compilation examples, codel size inference is being done implicitly.

### Nonterminating Piet programs.

I believe that some programs on the [piet samples](https://www.dangermouse.net/esoteric/piet/samples.html) page are a bit buggy, and a common mistake between these programs is nontermination.  Since Piet is Turing-complete, reasoning about arbitrary Piet program termination is equivalent to solving the halting problem; however, it is possible to enumerate a class of Piet programs that never terminate--the programs whose CFG nodes all have outdegree greater than 0.  This is explained [here](https://github.com/pwang00/pietcc/blob/main/Compiler.md#Termination).

Here are a few examples of this class of nonterminating Piet programs; running PietCC with the `-w` or `--warn-nt` flag warns users accordingly.

[Hello World 2](https://github.com/pwang00/pietcc/blob/main/images/hw2-11.gif)

<img src="https://github.com/pwang00/pietcc/blob/main/images/hw2-11.gif" alt="Buggy Hello World"/>

```
$ ./pietcc images/hw2-11.gif --warn-nt -o hw2-11
pietcc: warning: every node in program CFG has nonzero outdegree.  This implies nontermination!
```

[Hello World 5](https://github.com/pwang00/pietcc/blob/main/images/hw5_big.png)

<img src="https://github.com/pwang00/pietcc/blob/main/images/hw5_big.png" alt="Another Buggy Hello World"/>

```
$ ./pietcc images/hw5.png --warn-nt -o hw5
pietcc: warning: every node in program CFG has nonzero outdegree.  This implies nontermination!
```

## Generating control flow graph from Piet LLVM IR

Visualizing a CFG from LLVM IR can be helpful.  As an example, here's a program that simply push, pops, and dups.

[test2_upscaled.png](https://github.com/pwang00/pietcc/blob/main/images/test2_upscaled.png)

<img src="https://github.com/pwang00/pietcc/blob/main/images/test2_upscaled.png" alt="A sample program that does nothing"/>


To generate the CFG for that program, we can do 

```
$ ./pietcc images/test2.png -o test2 --emit-llvm
$ opt --dot-cfg test2.ll; dot .start.dot -Tpng > cfg.png
```

Which generates the CFG in PNG format.

Here's the relevant CFG for that program:

[cfg_test.png](https://github.com/pwang00/pietcc/blob/main/cfg_test.png)

<img src="https://github.com/pwang00/pietcc/blob/main/cfg_test.png" alt="CFG for above program" width=700/>

