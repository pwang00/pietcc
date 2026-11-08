# Compiling Piet to LLVM IR

## Control flow graph types

Piet control flow is modeled as a directed graph: each vertex represents a color block, and each edge groups the possible pointer states that transition to a neighboring block. The shared types live in [piet_core/src/cfg.rs](piet_core/src/cfg.rs) and [piet_core/src/flow.rs](piet_core/src/flow.rs). The main data layout is:

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub usize);

pub type Transitions = Vec<PietTransition>;
pub type NodeAdj = Vec<(BlockId, Transitions)>;

#[derive(Debug, Clone)]
pub struct BlockData {
    pub lightness: Lightness,
    pub count: usize,
    pub width: usize,
    pub height: usize,
    pub label: String,
}

#[derive(Debug)]
pub struct CFG {
    blocks: Vec<BlockData>,
    adjacencies: Vec<NodeAdj>,
    codel_size: usize,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PointerState {
    pub dp: DirPointer,
    pub cc: CodelChooser,
}

#[derive(Debug, Copy, Clone)]
pub struct PietTransition {
    pub entry_state: PointerState,
    pub exit_state: PointerState,
    pub instruction: Option<Instruction>,
}
```

`BlockId` is a dense index into the CFG's parallel block and adjacency vectors. IDs are assigned in discovery order, with `CFG::ENTRY = BlockId(0)` identifying the initial block. Consumers use `cfg.ids()`, `cfg.block(id)`, and `cfg.adjacencies(id)` to traverse the graph. Indexed lookup avoids string hashing, and ordered traversal makes the emitted IR reproducible.

Each `BlockData` contains:

* `lightness`: the block's Piet color.
* `count`: its size in codels, used as the value for `Push`.
* `width` and `height`: its bounding box dimensions in codels.
* `label`: a presentation name for diagnostics and LLVM basic blocks, formed from the color and the topmost, then leftmost source coordinate. Labels do not determine identity.

The CFG no longer retains a set of every coordinate in each block. Pixel membership belongs to the frontend's label raster; the graph keeps only the payload needed by interpretation, optimization, and lowering.

`NodeAdj` groups transitions by destination `BlockId`, preserving neighbor discovery order. Within each neighbor, transitions follow DP / CC order. Each `PietTransition` records the pointer state required to take the edge, the state on arrival, and an optional instruction. Direct colored-block transitions keep the same entry and exit state and decode the instruction from the color difference. Transitions traced through white execute no instruction and may change the pointer state.

## CFG generation

The [frontend pipeline](frontend/src/pipeline.rs) converts a `PietSource` raster into a CFG shared by the interpreter and compiler:

```text
Image file
    |
    v
CLI loader: decode pixels into a PietSource color raster
    |
    v
Lexer: two-pass connected-component labeling
    |  Scan equal-colored neighbors above and to the left;
    |  merge equivalent labels with union-find, then resolve roots.
    v
Parser: sweep the labeled raster
    |  Collect each component's color, pixel count, and eight exits
    |  (one per DP / CC state); derive bounding boxes from the exits.
    v
Resolve the top-left entry (trace first if white)
    |  Intern it as CFG::ENTRY; black or trapped white returns
    |  a single node with no outgoing transitions.
    v
Walk reachable components with a queue <---------------------+
    |                                                      |
    v                                                      |
Inspect all eight exits                                    |
    |                                                      |
    +-- Black / off canvas ----------> Skip                 |
    |                                                      |
    +-- Colored neighbor ------------> Decode instruction   |
    |                                      |               |
    +-- White --> Trace slide              |               |
                     |                     |               |
                     +-- No exit --> Skip  |               |
                     |                     |               |
                     +-- Colored exit -----+               |
                         (no instruction)  |               |
                                           v               |
                           Intern destination; group       |
                           transitions by BlockId;         |
                           queue newly discovered blocks --+
    |
    | Queue exhausted
    v
Choose codel size: explicit width, default 1, or inferred GCD
    |
    v
CFG::reinterpret: rescale counts and bounding boxes
    |
    v
CFG -> interpreter or compiler backend
```

The lexer's `DisjointSet` holds the source raster, a `Raster<usize>` of component labels, and union-find nodes. During parsing, each component has temporary `ComponentMetadata` with `lightness: Option<Lightness>`, `count: usize`, and `exits: [Position; 8]`. The exits are updated as pixels are scanned; their extreme coordinates also supply the bounding box and label anchor.

Labeling covers the entire image, while the queue walk only interns components reachable from the entry through valid exits. Block sizes initially use one source pixel per codel. Once the graph exists, the inference heuristic takes the GCD of its block widths and heights, or the pipeline uses the requested size. `CFG::reinterpret` rescales counts by codel area and dimensions by codel width without changing graph connectivity.

Black and off-canvas exits are omitted from the graph. Runtime retry logic handles those restrictions by changing DP / CC until an available transition matches. During lowering, [cfg_to_ir](backend/cfg_to_ir/) generates LLVM basic blocks from the resulting graph.

## White block tracing and elimination

White regions are traced during adjacency construction rather than added as ordinary graph nodes and removed in a separate pass. From an entry coordinate and pointer state, `trace_white` slides in the DP direction until it reaches a colored codel or a restriction. At a restriction, it rotates DP clockwise, toggles CC, and retries from the current coordinate, stopping after eight blocked attempts if no exit is found.

For a fixed entry coordinate and pointer state, tracing determines a single destination and arrival state, or reports no exit. If colored blocks A and C are connected through white region B, the parser emits A -> C directly. If tracing finds no exit, that candidate transition is omitted. A trapped white region at the program entry is retained as a terminal entry node.

Each successful white trace produces a `PietTransition` whose `entry_state` is the state used to leave A, whose `exit_state` is the state on reaching C, and whose `instruction` is `None`. No command is executed for the color difference across white. This avoids generating dispatch blocks for intermediate white regions.

## Code generation

PietCC targets LLVM IR for portability.  All LLVM IR code generation is done via [inkwell](https://github.com/TheDan64/inkwell); `llc` and `clang` are then used to convert the resulting IR into native assembly, and handle linking to libc to produce a final executable.  Lowering and output generation live in [backend/cfg_to_ir](backend/cfg_to_ir/). The following subsections describe the relevant functions; the IR listings are illustrative excerpts.

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
    * Piet stack of 64-bit integers.  This gets malloc'd at runtime
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

### Piet instructions

Compiled Piet instructions obey the spec [here](https://www.dangermouse.net/esoteric/piet.html).  Since Push and Dup increment the stack size, we need to make sure that the stack size is less than STACK_SIZE.  If the runtime stack exceeds this, the program terminates. Currently, the stack size is initialized as 

```rust
pub const STACK_SIZE: u32 = 1 << 18;
```

and the call to `malloc` allocates STACK_SIZE * sizeof(i64) = STACK_SIZE * 8 bytes.  Push and Dup both call `stack_size_check` before modifying the stack, which then calls `terminate` if the stack size is exceeded.  `terminate` then prints "Stack memory exhausted" and exits with code 1.

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

### CFG to IR

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

We know control flow has hit a restriction if the current value of the dp / cc doesn't satisfy any of the dp / cc checks for the current adjacency.  In this case, the retries counter `rctr` is incremented, the dp / cc are rotated depending on the parity of `rctr`, and control flow jumps back to the start of the current node's labels.  It follows that control flow will be able to exit the current node, since the runtime dp / cc will eventually be rotated to a state that satisfies one of the dp / cc comparisons.

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

A compiled Piet program terminates once a jump is taken to a color block that has no adjacencies.  In this case, `start` returns immediately and the stack state is printed.  It follows that every Piet program CFG that contains no nodes of outdegree 0 will never terminate, since only nodes of outdegree zero are compiled with `ret` instructions.  Therefore, we can detect a certain class of nonterminating programs at compile-time and warn the user accordingly.

### Optimizations

PietCC runs Piet-level passes from [backend/optimizer](backend/optimizer/) before lowering, then LLVM module passes through [cfg_to_ir's LLVM pipeline](backend/cfg_to_ir/src/llvm_pipeline.rs). The Piet pass manager provides abstractions for custom passes and cached analysis results.

Without an optimization flag, both stages are skipped. `--o1`, `--o2`, and `--o3` all attempt the same Piet static evaluation with a 200,000-step budget, then run LLVM's `default<O1>`, `default<O2>`, or `default<O3>` pipeline respectively. Higher levels apply more extensive LLVM optimizations and may cost more compilation time; they do not increase the Piet evaluation budget.

```rust
pub trait Pass: Debug {
    fn name(&self) -> &'static str;
    fn run(&mut self, cfg: &mut CFG, manager: &mut AnalysisCache) -> Result<(), Box<dyn Error>>;
}
```

Each pass can update its optimization result in the `AnalysisCache`.  Currently the `ExecutionResult` only stores the complete or partial execution states of a Piet program, but this can be extended to support CFG modifications.

```rust
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Complete(ExecutionState),
    Partial(ExecutionState),
}
```

```rust
#[derive(Debug, Clone)]
pub struct AnalysisCache {
    pub(crate) result: Option<ExecutionResult>,
}

impl AnalysisCache {
    pub fn get_cached_result(&self) -> Option<&ExecutionResult> {
        self.result.as_ref()
    }

    pub fn update_result(&mut self, result: ExecutionResult) {
        self.result = Some(result)
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self { result: None }
    }
}
```

The passes themselves are orchestrated by PietCC's `OptimizationPassManager`:

```rust
#[derive(Debug)]
#[allow(unused)]
pub struct OptimizationPassManager<'a> {
    passes: Vec<Box<dyn Pass>>,
    analysis_cache: AnalysisCache,
    settings: CompilerSettings<'a>,
}

impl<'a> OptimizationPassManager<'a> {
    pub fn run_all(&mut self, cfg: &mut CFG) {
        for pass in &mut self.passes {
            if let Err(err) = pass.run(cfg, &mut self.analysis_cache) {
                eprintln!("Error while running optimization pass: {}", err)
            }
        }
    }

    pub fn new(passes: Vec<Box<dyn Pass>>, settings: CompilerSettings<'a>) -> Self {
        Self {
            passes,
            analysis_cache: AnalysisCache::default(),
            settings,
        }
    }

    pub fn get_analysis_cache(&self) -> &AnalysisCache {
        &self.analysis_cache
    }
}
```

### Compile-time Constant Evaluation

Compile-time evaluation provides enormous runtime and compilation benefits.  For large programs, LLVM IR verification can take an extremely long time, so running the Piet interpreter on the program to obtain a final execution result with the stack and stdout and compiling those as printable constants can drastically reduce compilation times and increase runtime efficiency.  The pass is implemented as follows:

```rust
#[derive(Debug)]
pub struct StaticEvaluatorPass;

pub const MAX_STEPS: u64 = 200000;

impl Pass for StaticEvaluatorPass {
    fn name(&self) -> &'static str {
        "static_eval"
    }

    fn run(
        &mut self,
        cfg: &mut CFG,
        analysis_cache: &mut AnalysisCache,
    ) -> Result<(), Box<dyn Error>> {
        let codel_settings = piet_core::settings::CodelSettings::Default;
        let static_eval_settings =
            StaticEvaluatorSettings::abstract_interp(MAX_STEPS, codel_settings);
        let mut static_eval = StaticEvaluator::new(cfg, static_eval_settings);
        let execution_state = static_eval.run();
        match execution_state.status {
            ExecutionStatus::Completed => {
                Ok(analysis_cache.update_result(ExecutionResult::Complete(execution_state)))
            }
            ExecutionStatus::MaxSteps => {
                Ok(analysis_cache.update_result(ExecutionResult::Partial(execution_state)))
            }
            ExecutionStatus::NeedsInput => Err(Box::new(OptimizerError::StaticEvaluationError(
                "Static evaluation on input-dependent programs not yet implemented.  Will compile with partial execution result.".into(),
                ExecutionResult::Partial(execution_state),
            ))),
            _ => Ok(()),
        }
    }
}
```

#### Constant programs

The compile-time evaluator can completely fold constant programs that finish within its step budget without requiring input.  Let's consider the example of [Hello World 1:](https://github.com/pwang00/pietcc/blob/main/images/hw1-11.gif)

<img src="https://github.com/pwang00/pietcc/blob/main/images/hw1-11.gif" alt="Hello World with Codel Size 11"/>

Here's what an LLVM IR file might look like for an unoptimized version of the program:

```
; all instruction definitions

define i64 @main() {
  call void @init_globals()
  %malloc = call ptr @malloc(i64 2097152)
  store ptr %malloc, ptr @piet_stack, align 8
  call void @set_stdout_unbuffered()
  call void @start()
  call void @print_piet_stack()
  ret i64 0
}

define void @start() {
  br label %Entry

LightRed_5_9:                                     ; preds = %call_instr_LightYellow_6_91524, %call_instr_LightYellow_6_91523, %call_instr_RegRed_5_101062, %call_instr_RegRed_5_101061, %call_instr_DarkRed_4_9260, %call_instr_DarkRed_4_9, %rotate_pointers_LightRed_5_9
  %load_dp = load i8, ptr @dp, align 1
  %load_cc = load i8, ptr @cc, align 1
  br label %adjacency_LightRed_5_9_0

adjacency_LightRed_5_9_2:                         ; preds = %dirvec_adj4
  br label %dirvec_adj7

rotate_pointers_LightRed_5_9:                     ; preds = %dirvec_adj8
  call void @retry()
  br label %LightRed_5_9

dirvec_adj8:                                      ; preds = %dirvec_adj7
  %1 = icmp eq i8 %load_dp, 3
  %2 = icmp eq i8 %load_cc, 1
  %3 = and i1 %1, %2
  br i1 %3, label %call_instr_LightRed_5_910, label %rotate_pointers_LightRed_5_9

...

<hundreds of more basic blocks>
}
```

whereas a fully folded version simply contains both the stack and stdout states as string literals, which are printed at runtime.

```
; ModuleID = 'piet'
source_filename = "piet"

@stack_id_empty = private unnamed_addr constant [13 x i8] c"\0AStack empty\00", align 1
@stack_vals = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str = private unnamed_addr constant [14 x i8] c"Hello, world!\00", align 1

; Function Attrs: nofree nounwind
declare noundef i32 @printf(ptr nocapture noundef readonly, ...) local_unnamed_addr #0

; Function Attrs: nofree nounwind
define noundef i64 @main() local_unnamed_addr #0 {
print_stdout:
  %puts = tail call i32 @puts(ptr nonnull dereferenceable(1) @str)
  %print_stack = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @stack_id_empty, ptr nonnull @stack_vals)
  ret i64 0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define void @init_globals() local_unnamed_addr #1 {
  ret void
}

; Function Attrs: nofree nounwind
declare noundef i32 @puts(ptr nocapture noundef readonly) local_unnamed_addr #0

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
```

#### Non-Terminating or Input-Dependent Programs

The compile-time evaluator cannot fold input-dependent or otherwise non-terminating programs, but can still generate programs that can "resume" from a given state, which may reduce the number of instructions executed at runtime. To be precise, the evaluator runs until either `MAX_STEPS` is reached, or a `CharIn` or `IntIn` instruction is encountered.  The stack, stdout, dp, cc states until this point are compiled into the program, and the saved block ID determines where the `start` function resumes.  Using the above Hello World program as an example, a completely unoptimized variant might produce

```
define i64 @main() {
  call void @init_globals()
  %malloc = call ptr @malloc(i64 2097152)
  store ptr %malloc, ptr @piet_stack, align 8
  call void @set_stdout_unbuffered()
  call void @start()
  call void @print_piet_stack()
  ret i64 0
}

define void @start() {
  br label %Entry
  ...
}
```

but if the evaluator were to stop after a certain number of steps, we would have 

```
define i64 @main() {
  call void @init_globals()
  %malloc = call ptr @malloc(i64 2097152)
  store ptr %malloc, ptr @piet_stack, align 8
  call void @set_stdout_unbuffered()
  call void @initialize_piet_stack()
  call void @start()
  call void @print_piet_stack()
  ret i64 0
}

define void @initialize_piet_stack() {
initialize_piet_stack_body:
  store i64 1, ptr @stack_size, align 4
  %call_printf_stdout = call i32 (ptr, ...) @printf(ptr @stdout_state)
  %piet_stack_load = load ptr, ptr @piet_stack, align 8
  %index = getelementptr i64, ptr %piet_stack_load, i64 0
  store i64 10, ptr %index, align 4
  ret void 
}

define void @start() {
  br label %LightCyan_8_12
  ...
}
```

The key differences are that we compile the stack and stdout states into the program and print stdout before jumping to the correct adjacency.  Note that in the partially-evaluated `start` function, we jump to `%LightCyan_8_12` instead of `Entry`.
