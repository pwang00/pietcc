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

```Rust
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

PietCC targets LLVM IR for sake of portability.  All LLVM IR code generation is done via [inkwell](https://github.com/TheDan64/inkwell); `llc` and `clang` are then used to convert the resulting IR into native assembly, and handle linking to libc to produce a final executable.  The following subsections go into more detail about the relevant functions.

### Libc functions

* `declare i64* @malloc(i64)`
* `declare i64 @printf(i8*, ...)`
* `declare i64 @__isoc99_scanf(i8*, ...)`
* `declare void @setvbuf(i8*, i8*, i32, i32)`
* `declare i8* @fdopen(i32, i8*)`

`setvbuf` and `fdopen` are used for setting stdout to unbuffered.

```llvm
define void @set_stdout_unbuffered() {
  %stdout = call i8* @fdopen(i32 1, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @fdopen_mode, i32 0, i32 0))
  call void @setvbuf(i8* %stdout, i8* null, i32 2, i32 0)
  ret void
}
```

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

The stack is initialized by `init_globals`, and we get the kth element from the top by doing `stack[stack_size - 1 - k]`.

```llvm
define void @init_globals() {
  %malloc = call i64* @malloc(i64 1048576)
  store i64* %malloc, i64** @piet_stack, align 8
  ret void
}
```

There are also a lot of format strings for `printf` and `scanf`, such as `"%s", "%d"`, etc.  These aren't super important.

### Piet instructions

All Piet instructions are compiled into each program, and they follow spec [here](https://www.dangermouse.net/esoteric/piet.html).  Since Push and Dup increment the stack size, we need to make sure that the stack size is less than the constant size of `STACK_SIZE`.  If the runtime stack exceeds STACK_SIZE, then the program terminates. Currently, the stack size is initialized as 

```rust
pub const STACK_SIZE: u32 = 1 << 18;
```

and the call to `malloc` allocates STACK_SIZE * sizeof(i64) = STACK_SIZE * 8 bytes.  Push and Dup both call `stack_size_check` before modifying the stack, which then calls `terminate`, which prints "Stack memory exhausted" and exits with code 1.

```
define void @piet_push(i64 %0) {
  call void @stack_size_check()
  ...
}

define void @piet_dup() {
  call void @stack_size_check()
  ...
}

define void @stack_size_check() {
  %stack_size = load i64, i64* @stack_size, align 4
  %check_overflow = icmp uge i64 %stack_size, 262144
  br i1 %check_overflow, label %1, label %2

1:                                                ; preds = %0
  %call_terminate = call i64 @terminate()
  unreachable

2:                                                ; preds = %0
  ret void
}

define i64 @terminate() {
  %load_exhausted_fmt = load [46 x i8], [46 x i8]* @exhausted_fmt, align 1
  %1 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([46 x i8], [46 x i8]* @exhausted_fmt, i64 0, i64 0))
  call void @print_piet_stack()
  %call_exit = call i64 @exit(i64 1)
  ret i64 1
}
```

### Program entrypoint

```llvm
define i64 @main() {
  call void @init_globals()
  call void @set_stdout_unbuffered()
  call void @start()
  call void @print_piet_stack()
  ret i64 0
}
```

`start` marks the start of the actual Piet program.

### Control flow graph to IR

For every node in our CFG with outdegree greater than 0, we generate the LLVM basic blocks with the following structure:
```
Node entry
        |
    Node adjacency
            |
        Node bordering direction
                |
            Call instruction if dp and cc match and jump to next node
                |
        Next node direction
                |
            Call instruction if dp and cc match and jump to next node
                |
    Next node adjacency
            |
        Node bordering direction
                |
            Call instruction if dp and cc match and jump to next node
            ...
            |
            ...
    rotate pointers
    jump to node entry
```

Otherwise we just generate

```
Node entry
    | 
    jump to ret
```

Here's an example of what the generated IR might look like:

```llvm
Entry:                                            ; preds = %rotate_pointers_Entry, %call_instr_DarkRed_5_34, %0
  %load_dp22 = load i8, i8* @dp, align 1
  %load_cc23 = load i8, i8* @cc, align 1
  br label %adjacency_Entry_0

adjacency_Entry_0:                                ; preds = %Entry
  br label %dirvec_adj24

rotate_pointers_Entry:                            ; preds = %dirvec_adj25
  call void @retry()
  br label %Entry

dirvec_adj25:                                     ; preds = %dirvec_adj24
  %28 = icmp eq i8 %load_dp22, 0
  %29 = icmp eq i8 %load_cc23, 1
  %30 = and i1 %28, %29
  br i1 %30, label %call_instr_Entry26, label %rotate_pointers_Entry

call_instr_Entry26:                               ; preds = %dirvec_adj25
  call void @piet_push(i64 12)
  store i8 0, i8* @rctr, align 1
  br label %DarkRed_5_3

dirvec_adj24:                                     ; preds = %adjacency_Entry_0
  %31 = icmp eq i8 %load_dp22, 0
  %32 = icmp eq i8 %load_cc23, 0
  %33 = and i1 %31, %32
  br i1 %33, label %call_instr_Entry, label %dirvec_adj25

call_instr_Entry:                                 ; preds = %dirvec_adj24
  call void @piet_push(i64 12)
  store i8 0, i8* @rctr, align 1
  br label %DarkRed_5_3
```

Every `dirvec` label represents the node bordering direction with its adjacency, in which the current dp / cc during runtime are compared with the statically determined bordering directions.  If the comparison evaluates to true, then the corresponding instruction is executed, the program retries counter `rctr` is reset, and a jump is taken to the adjacency.

### Restrictions

We know control flow has hit a restriction if the current value of the dp / cc doesn't satisfy any of the dp / cc checks for the current adjacency.  In this case, the retries counter `rctr` is incremented, the dp / cc are rotated depending on the parity of `rctr`, and control flow jumps back to the start of the current node's labels.  It follows that control flow will be able to exit the current node, since the runtime dp / cc will be eventually be rotated to a state that satisfies one of the dp / cc comparisons. 

This dp / cc rotation behavior is implemented in `retry`, shown below:

```llvm
define void @retry() {
  %load_dp = load i8, i8* @dp, align 1
  %load_cc = load i8, i8* @cc, align 1
  %load_rctr = load i8, i8* @rctr, align 1
  %1 = urem i8 %load_rctr, 2
  %2 = icmp eq i8 %1, 1
  br i1 %2, label %one_mod_two, label %zero_mod_two

one_mod_two:                                      ; preds = %0
  %rotate_dp = add i8 %load_dp, 1
  %dp_mod_4 = urem i8 %rotate_dp, 4
  store i8 %dp_mod_4, i8* @dp, align 1
  br label %ret

zero_mod_two:                                     ; preds = %0
  %rotate_cc = add i8 %load_cc, 1
  %dp_mod_41 = urem i8 %rotate_cc, 2
  store i8 %dp_mod_41, i8* @cc, align 1
  br label %ret

ret:                                              ; preds = %zero_mod_two, %one_mod_two
  %3 = add i8 %load_rctr, 1
  %4 = urem i8 %3, 8
  store i8 %4, i8* @rctr, align 1
  ret void
}
```

### Termination

A compiled Piet program terminates once a jump is taken to a color block that has no adjacencies.  In this case, `start` returns immediately and the stack state is printed.  It follows that every Piet program CFG that contains no nodes of outdegree 0 will never terminate, since only nodes of oudegree zero are compiled with `ret` instructions.  Therefore, we can detect a certain class of nonterminating programs at compile-time and warn the user accordingly.