# Compiling Piet to LLVM IR

## Control flow graph types

We can model Piet control flow like a directed graph: each vertex represents a color block and each edge represents a transition between color blocks, which encodes the command to be executed.  We can consider the following types to represent our control flow graph: 

```Rust
pub type DirVec = (Direction, Codel);
pub(crate) type Node = Rc<ColorBlock>;
pub(crate) type Info = Vec<(EntryDir, ExitDir, Option<Instruction>)>;
pub(crate) type Adjacencies = HashMap<Node, Info>;
pub(crate) type CFG = HashMap<Node, Adjacencies>;

#[allow(unused)]
pub struct CFGGenerator<'a> {
    program: &'a Program<'a>,
    adjacencies: CFG,
    codel_width: u32,
}

#[allow(unused)]
#[derive(Eq)]
pub(crate) struct ColorBlock {
    label: String,
    lightness: Lightness,
    region: HashSet<Position>,
}
```

The RC stuff is just so we don't have to clone the contents of the ColorBlock every time we want to insert into the map, but admittedly this is really ugly and probably not at all a good practice so I'll see if I can think of a better solution.

In a `Node`:
* `label` is a string consisting of `{current color}_{minimum block row}_{minimum block col}`.  This is done because our adjacencies are stored in a hashset using the label as a hash for efficiency, so we don't want to double-store blocks that have identical regions but different labels.
* `lightness` stores the current color
* `region` is a set of all coordinates in the color block

`Info = Vec<(EntryDir, ExitDir, Option<Instruction>)>` represents the adjacency data for each node.  In particular, `(EntryDir, ExitDir, Option<Instruction>)` represents, respectively, the current direction state (direction pointer, codel chooser), (direction pointer, codel chooser) after a potential transition, and command encoded by the color difference between the current node and adjacency.  This command isn't necessarily going to be `Some` even between two non-white adjacencies in the final CFG, which will be explained in the white block elimination section.

The rest is pretty straightforward: `Adjacencies = HashMap<Node, Info>` is a map of every node with its adjacency data, and `CFG = HashMap<Node, Adjacencies>` is the adjacency list representation for our entire program's control flow graph.

## Control flow graph generation

Generating a CFG for Piet can be done in the following steps:

1. Discover all pixels in the current color block via BFS.  
2. Determine all possible exits from the current color block, and enqueue the unvisited ones.  Note that we filter out all exits that are either black or out of bounds.
3. Iterate through the remaining coordinates in the boundaries and filter out the visited ones.  This is important since otherwise we might be doing repeated work trying to discover the same color block.
4. For non-white blocks, discover each adjacent color block corresponding the block's exits, determine the bordering direction, corresponding instruction to be executed, and add the node and its adjacencies to the CFG.  Note that in the context of `Info`

```rust
(EntryDir, ExitDir, Option<Instruction>);
```

We set the `EntryDir` and `ExitDir` to be the same, since from a control-flow perspective, only hitting restrictions or tracing white blocks can change the dp / cc, and we don't explicitly represent restrictions in our CFG (if a block's exit is out of bounds or black, we simply don't add it).  We set the `Instruction` to be the corresponding one based on the lightness / hue differences between the current and adjacent block colors.  During compilation, we add an LLVM basic block for each 


## White block tracing and elimination

White blocks follow a different exit convention than blocks of other color.  Namely, instead of selecting an exit codel based on the furthest direction in dp / cc, white blocks require moving in the direction of dp until a non-white or non-black block is hit and rotating the dp / cc upon collision with a restriction.  However, the exits can be determined statically by simply tracing from all possible entry points into the white block with the correct dp / cc, which are fixed by the adjacent block.  Furthermore, it's easy to see that an exit from a white block from a given entry and direction is unique, since if one exists, then it is necessarily the first non-white codel reached while traveling in the direction of dp.  

Once the exits have been traced, we can eliminate white blocks entirely from our CFG and join the blocks corresponding to the entry point and exit point with an edge.  As an example, let A, C be non-white blocks and let B be a white block.  Then after elimination, A -> B -> C becomes A -> C if it's determined that B can be exited from A into C with the given adjacency state.  Otherwise, if there is no way out from B, we would just have A -> B becomes A.

By eliminating white blocks, we can simplify our CFG and eliminate the need to generate a label, list of branches containing all possible dp / cc states for entrance, and a jump for every white block.  In the context of `Info`,

```rust
(EntryDir, ExitDir, Option<Instruction>)
```

We set the `EntryDir` to be the entry direction state, and `ExitDir` to be exit direction state, and `Instruction` to be `None` since no command is executed between white block transitions.

## Code generation

PietCC targets LLVM IR for sake of portability.  All LLVM IR code generation is done via [inkwell](https://github.com/TheDan64/inkwell); `llc` and `clang` are then used to convert the resulting IR into native assembly, and handle linking to libc to produce a final executable.  The following subsections go into more detail about the relevant functions / globals

### Libc functions

* `declare i64* @malloc(i64)`
* `declare i64 @printf(i8*, ...)`
* `declare i64 @__isoc99_scanf(i8*, ...)`
* `malloc`
* `setvbuf`
* `fdopen`

### LLVM intrinsics

* `@llvm.stackrestore`
* `@llvm.stacksave`
* `@llvm.smax.i64`

### Globals

* `@piet_stack = internal global i64* null`
    * Piet stack of 64 bit integers.  This gets malloc'd at runtime
* `@dp = internal global i8 0`
    * Direction pointer
* `@cc = internal global i8 0`
    * Codel chooser
* `@stack_size = internal global i64 0`
    * Stack size, used for indexing into the stack
* `@rctr = internal global i8 0`
    * Retries counter (this differs from the interpreter in that it's used for parity checking only when deciding whether to increment the dp or cc)

The stack is initialized by `init_globals`:

```llvm
define void @init_globals() {
  %malloc = call i64* @malloc(i64 1048576)
  store i64* %malloc, i64** @piet_stack, align 8
  ret void
}
```

### Program entrypoint

* `start`

### Hitting restrictions

A comoiok

### Termination

A compiled Piet program terminates once a jump is taken to a color block that has no adjacencies.  In this case, the function returns immediately and the final stack is printed.

