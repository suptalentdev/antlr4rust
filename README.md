# antlr4rust
ANTLR4 runtime for Rust programming language 

Generator part is currently located in rust-target branch of my antlr4 fork [rrevenantt/antlr4/tree/rust-target](https://github.com/rrevenantt/antlr4/tree/rust-target)

For examples you can see [grammars](grammars), [tests/gen](tests/gen) for corresponding generated code 
and [tests/my_tests.rs](tests/my_test.rs) for actual usage examples 

### Implementation status

Everything is implemented, "business" logic is quite stable and well tested, but user facing 
API is not very robust yet an very likely will have some changes.

For now development is going on in this repository 
but eventually it will be merged to main ANTLR4 repo

Currently requires nightly version of rust. 
This very likely will be the case until `specialization`,`try_blocks` and `unsize` features are stabilized. 

Remaining things before merge:
 - API stabilization
   - [ ] Rust api guidelines compliance  
   - [ ] more tests for API because it is quite different from Java
 - make parsing zero copy(i.e. use &str(or Cow) instead String in token and &Token in tree nodes)
 - more generic `PredictionContext`
 - generic over ownership for string
 - profiling and performance optimizations
 - generate enum for labeled alternatives without redundant `Error` option
 - ? option to generate fields instead of getters by default

Can be done after merge: 
 - Documentation
   - [ ] Quite some things are already documented but still far from perfect
 - Code quality
   - [ ] Rustfmt fails to run currently
   - [ ] Clippy sanitation 
   - [ ] Not all warning are fixed
 - visitor
 - build.rs integration + example
 - run rustfmt on generated parser
###### Long term improvements
 - use & instead of Rc for nodes in parser
 (requires GAT, otherwise it would be a problem for users that want ownership for parse tree)
 - support stable rust
 - support no_std(although alloc would still be required)  
  
### Usage

Add to `Cargo.toml`
```toml 
[dependencies]
lazy_static = "1.4"
antlr-rust = "0.1"
```
and `#![feature(try_blocks)]` in your project root module
 
### Parse Tree structure

It is possible to generate idiomatic Rust syntax trees. For this you would need to use labels feature of ANTLR.
You can see [Labels](grammars/Labels.g4) grammar for example.
Consider following rule :
```text
e   : a=e op='*' b=e   # mult
    | left=e '+' b=e   # add
		 
```
For such rule ANTLR will generate enum `EContextAll` containing `mult` and `add` alternatives, 
so you will be able to match on them in your code. 
Also corresponding struct for each alternative will contain fields you labeled. 
I.e. for `MultContext` struct will contain `a` and `b` fields containing child subtrees and 
`op` field with `TerminalNode` type which corresponds to individual `Token`.
It also is possible to disable generic parse tree creation to keep only selected children via
`parser.build_parse_trees = false`.
  
### Differences with Java
Although Rust runtime API is made as close as possible to Java, 
there are quite some differences because Rust is not an OOP language and is much more explicit. 

 - All rule context variables (rule argument or rule return) should implement `Default + Clone`.
 - If you are using labeled alternatives, 
 struct generated for rule is a enum with variant for each alternative
 - Parser needs to have ownership for listeners, but it is possible te get listener back via `ListenerId`
 otherwise `ParseTreeWalker` should be used.
 - In embedded actions to access parser you should use `recog` variable instead of `self`. 
 This is because predicate have to be inserted into two syntactically different places in generated parser 
 
 
### Unsafe
Currently unsafe is used only to cast from trait object back to original type 
and to update data inside Rc via `get_mut_unchecked`(returned mutable reference is used immediately and not stored anywhere)
  
## Licence

BSD 3-clause 
