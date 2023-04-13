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
2. Determine all possible exits from the current color block, and enqueue the ones that haven't already been discovered.
3. Iterate through the remaining coordinates in the boundaries and filter out the ones that are contained in B's region.  This is important since otherwise we might be doing repeated work trying to discover the same color block.
4. Discover each color block corresponding to distinct boundaries in A's boundary set, and enqueue these.

## Code Generation (Idea)

We represent every color block as its own function, referencing the global stack, direction pointer, and codel chooser. The CFG encodes both the color of the block and possible exits, and thus also the command to be executed.  Generally speaking

* A global stack depth, equivalent to number of elements in the stack * sizeof(i64) = 8
* A global Piet stack, allocated via malloc.  Basically due to the way malloc works, it's easiest to just push an element at *(stack + stack_depth * 8) and pop by doing stack_depth - 8
* A global direction pointer
* A global codel chooser
* An array of function pointers corresponding to Piet instructions
* A function for every color block.

As far as I'm aware, this can be done with inkwell.

