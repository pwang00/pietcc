# Compiling Piet to LLVM IR

## Control Flow Graph types

We can model Piet control flow like a directed graph: each vertex represents a color block and each edge represents a transition between color blocks, which encodes the command to be executed.  We can consider the following types to represent our control flow graph: 

```Rust
pub type DirVec = (Direction, Codel);
pub(crate) type Node = Rc<ColorBlock>;
pub(crate) type Info = Vec<(DirVec, DirVec, Option<Instruction>)>;
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

The RC stuff is just so we don't have to clone the contents of the ColorBlock every time we want to insert into the map, but admittedly this is really ugly so I'll see if I can think of a better solution.

In a `Node`:
* `label` is a string consisting of `{current color}_{minimum block row}_{minimum block col}`.  This is done because our adjacencies are stored in a hashset using the label as a hash for efficiency, so we don't want to double-store blocks that have identical regions but different labels.
* `lightness` stores the current color
* `region` is a set of all coordinates in the color block

`Info` represents the adjacency data for each node, namely the current direction state (direction pointer, codel chooser), (direction pointer, codel chooser) after a potential transition, and command encoded by the color difference between the current node and adjacency.  This command isn't necessarily going to be `Some` even between two non-white adjacencies, which will be explained in the white block elimination section.

The rest is pretty straightforward: `Adjacencies` is a map of every node with its adjacency data, and `CFG` is the adjacency list representation for our entire program's control flow graph.

## Control Flow Graph generation

Generating a CFG for Piet can be done in the following steps:

1. Discover all pixels in the current color block via BFS.  
2. Determine all possible exits from the current color block, and enqueue the unvisited ones.
3. Iterate through the remaining coordinates in the boundaries and filter out the visited ones.  This is important since otherwise we might be doing repeated work trying to discover the same color block.
4. For non-white blocks, discover each adjacent color block corresponding the block's exits, determine the bordering direction, corresponding instruction to be executed, and enqueue the ones that haven't already been discovered.

## White Block Tracing and Elimination

White blocks follow a different exit convention than blocks of other color.  Namely, instead of selecting an exit codel based on the furthest direction in dp / cc, white blocks require moving in the direction of dp until a non-white or non-black block is hit, and rotating the dp / cc upon collision with a restriction.  However, the exits can be determined statically by simply tracing from all possible entry points into the white block with the correct dp / cc, which are fixed by the adjacent block.  Furthermore, an exit from a white block from a given entry and direction is unique, since if an exit 

## Code Generation

We represent every color block as its own basic block with labels, referencing the global stack, direction pointer, and codel chooser. The CFG encodes both the color of the block and possible exits, and thus also the command to be executed.  

* A global stack depth, equivalent to number of elements in the stack * sizeof(i64) = 8
* A global Piet stack, allocated via malloc.  Basically due to the way malloc works, it's easiest to just push an element at *(stack + stack_depth * 8) and pop by doing stack_depth - 8
* A global direction pointer
* A global codel chooser
* A sequence of basic blocks corresponding to every color block.


