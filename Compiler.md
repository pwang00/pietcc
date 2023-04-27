# Compiling Piet to LLVM IR

## Control Flow

We can model Piet control flow like a directed graph: each vertex represents a color block and each edge represents a transition between color blocks, which encodes the command to be executed.  We can consider the following structures to represent our control flow graph: 

```Rust
pub(crate) type Node = Rc<ColorBlock>;
pub(crate) type Adjacencies = HashMap<Node, Vec<DirVec>>;
pub(crate) type CFG = HashMap<Node, Adjacencies>;

#[allow(unused)]
pub struct CFGGenerator<'a> {
    program: &'a Program<'a>,
    adjacencies: CFG,
    codel_width: u32,
}

#[allow(unused)]
#[derive(Debug, Eq)]
pub(crate) struct ColorBlock {
    label: String,
    lightness: Lightness,
    region: HashSet<Position>,
}
```

The RC stuff is just so we don't have to clone the contents of the ColorBlock every time we want to insert into the map, but admittedly this is really ugly so I'll see if I can think of a better solution.

In a `ColorBlock`:
* `label` is a string consisting of `{current color}_{minimum row}_{minimum col}`.  This is done because our adjacencies are stored in a hashset using the label as a hash for efficiency, so we don't want to double-store blocks that have identical regions but different labels.
* `lightness` stores the current color
* `region` is a set of all coordinates in the color block

Generating a CFG for Piet can be done in the following steps:

1. Discover all pixels in the current color block via BFS.  
2. Determine all possible exits from the current color block, and enqueue the ones that haven't already been discovered.
3. Iterate through the remaining coordinates in the boundaries and filter out the ones that are contained in B's region.  This is important since otherwise we might be doing repeated work trying to discover the same color block.
4. Discover each color block corresponding to distinct exits of A, and enqueue these.

## White Block Tracing

White blocks follow a different exit convention than blocks of other color--namely, instead of selecting an exit codel based on the furthest direction in dp / cc, white blocks require moving in the direction of dp until a non-white or non-black block is hit, and rotating the dp / cc upon collision with a restriction.  However, the exits (or lack thereof) can be determined statically by simply tracing from all possible entry points into the white block with the correct dp / cc, which are fixed by the adjacent block.  The interpreter already has this implemented, so getting this working with the compiler shouldn't take too long.

## Code Generation

We represent every color block as its own basic block with labels, referencing the global stack, direction pointer, and codel chooser. The CFG encodes both the color of the block and possible exits, and thus also the command to be executed.  

* A global stack depth, equivalent to number of elements in the stack * sizeof(i64) = 8
* A global Piet stack, allocated via malloc.  Basically due to the way malloc works, it's easiest to just push an element at *(stack + stack_depth * 8) and pop by doing stack_depth - 8
* A global direction pointer
* A global codel chooser
* A sequence of basic blocks corresponding to every color block.


