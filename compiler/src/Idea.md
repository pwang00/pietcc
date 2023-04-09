# Compiling Piet to x86_64

## Control Flow

We can model Piet control flow like a directed graph: each vertex represents a color block and each edge represents a transition between color blocks, which encodes the command to be executed.  We can consider the following structures to represent our control flow graph: 

```Rust
struct ColorBlock {
    label: String,
    lightness: Lightness,
    region: HashSet<Position>,
}

type DirVec = (Direction, Codel)
type AdjacencyList = HashMap<Rc<ColorBlock>, HashMap<Rc<ColorBlock>, Vec<DirVec>>>;

pub struct CFGGenerator<'a> {
    program: &'a Program<'a>,
    adjacencies: AdjacencyList,
}
```

The RC stuff is just so we don't have to clone the contents of the ColorBlock every time we want to insert into the map, but admittedly this is really ugly so I'll see if I can think of a better solution.

In a `ColorBlock`:
* `label` is a string consisting of `current color, entry point row + col`
* `lightness` is a string consisting of the current color
* `region` is a set of all coordinates in the color block

Generating a CFG for Piet can be done in the following steps:

1. Discover all pixels in the current color block via BFS.  
2. Once discovery is done, select some random coordinate from the boundary set and explore it.  Call the resulting color block B.
3. Iterate through the remaining coordinates in the boundaries and filter out the ones that are contained in B's region.  This is important since otherwise we might be doing repeated work trying to discover the same color block.
4. Discover each color block corresponding to distinct boundaries in A's boundary set, and enqueue these.

## Code Generation

We represent every color block as its own label.  The CFG encodes both the color of the block and possible exits, and thus also the command to be executed.  

We want the following:

* A global dynamically allocated stack to act as the Piet stack.  We can build an LLVM function signature matching libc malloc and call it to generate the runtime stack.

## Calling convention

Maintain the following registers: 

* `rsi`: pointer to dynamically allocated Piet stack 
* `rax`: direction pointer state ({0 ... 4} for right, down, left, up).
* `rbx`: codel chooser state ({0, 1} for left, right)
* `rcx`: address of current color block (label address should be known at compile-time)
* `r8`: color of previous color block
* `r9`: color of current color block
* `r10`: number of codels in a color block.
* `r11`: Piet stack depth (so we can add back the correct offset to align the stack after upon termination, if termination occurs).
* `r12`: retries counter (if we hit 8, then terminate the program)
* `r13`: old label address (we need this to determine whether or not to increment the retries counter)

Everything else can be just pushed / popped from the stack, though I suspect handling edge cases such as there not being enough operands on the stack will be pretty annoying.

Notes:

* The color of each block (discarding white and black) can be represented by some value in Zmod(18), so that their differences, which encode the command to be executed, are also in that range.  
* We can compile an auxiliary function that decodes the command based on `r9 - r8` and chooses which command to call.

# Execution

As an example, consider a small square Piet program with only 4 color blocks corresponding to vertices `v_1 ... v_4`, like so:

![](example.png)

So 

* `v_1` is bordered by `v_2` and `v_3` 
* `v_2` is bordered by `v_1` and `v_4`
* `v_3` is bordered by `v_1` and `v_4`
* `v_4` is bordered by `v_2` and `v_3`.  

Thus, we can assign some label name to each `v_i`, and output code to jump to a certain adjacency of `v_i` depending on the values of dp / cc, or back to itself if none of the values are valid.  For example, for `v_2`, we could output the following asm: 

```
label_v2:
  sub rdx, r9, r8
  mov r12, 0

  ; Casework on cc
  ; 0 for left, 1 for right

  cmp rbx, 0
  je label_v2_dp
  cmp rbx, 1
  je label_v2_dp

label_v2_invalid:
  call increment_retries_counter
  jmp label_v2

label_v2_dp:
  ; Casework on dp
  ; v_2 has two adjacencies: one to the left and one below.
  cmp rax, 1
  jmp label_v1
  cmp rax, 2
  jmp label_v4
```

