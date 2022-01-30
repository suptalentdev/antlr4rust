# antlr4rust
[![docs](https://docs.rs/antlr-rust/badge.svg)](https://docs.rs/antlr-rust)
[![Crate](https://img.shields.io/crates/v/antlr_rust.svg)](https://crates.io/crates/antlr_rust)

[ANTLR4](https://github.com/antlr/antlr4) runtime for Rust programming language.

Tool(generator) part is currently located in rust-target branch of my antlr4 fork [rrevenantt/antlr4/tree/rust-target](https://github.com/rrevenantt/antlr4/tree/rust-target)
Latest version is automatically built to [releases](https://github.com/rrevenantt/antlr4rust/releases) on this repository.
Also you can checkout it and `mvn -DskipTests install`

For examples you can see [grammars](grammars), [tests/gen](tests/gen) for corresponding generated code 
and [tests/my_tests.rs](tests/my_test.rs) for actual usage examples

### Implementation status

For now development is going on in this repository 
but eventually it will be merged to main ANTLR4 repo

Currently, requires nightly version of rust. 
This likely will be the case until `coerce_unsize` or some kind of coercion trait is stabilized. 
There are other unstable features in use but only `CoerceUnsized` is essential. 

Remaining things before merge:
 - API stabilization
   - [ ] Rust api guidelines compliance  
   - [ ] more tests for API because it is quite different from Java

Can be done after merge: 
 - more profiling and performance optimizations
 - Documentation
   - [ ] Some things are already documented but still far from perfect, also more links needed.
 - Code quality
   - [ ] Clippy sanitation 
   - [ ] Not all warning are fixed
 - cfg to not build potentially unnecessary parts 
 (no Lexer if custom token stream, no ParserATNSimulator if LL(1) grammar)  
 - run rustfmt on generated parser
###### Long term improvements
 - generate enum for labeled alternatives without redundant `Error` option
 - option to generate fields instead of getters by default and make visiting based on fields
 - make tree generic over pointer type and allow tree nodes to arena.
 (requires GAT, otherwise it would be a problem for users that want ownership for parse tree)
 - support stable rust
 - support no_std(although alloc would still be required)  
  
### Usage

You should use the ANTLR4 "tool" to generate a parser, that will use the ANTLR 
runtime, located here. You can run it with the following command:
```bash
java -jar <path to ANTLR4 tool> -Dlanguage=Rust MyGrammar.g4
```
For a full list of antlr4 tool options, please visit the 
[tool documentation page](https://github.com/antlr/antlr4/blob/master/doc/tool-options.md).

You can also see [build.rs](build.rs) as an example of `build.rs` configuration 
to rebuild parser automatically if grammar file was changed.

Then add following to `Cargo.toml` of the crate from which generated parser 
is going to be used:
```toml 
[dependencies]
antlr-rust = "=0.2.0-dev.1"
```
and `#![feature(try_blocks)]` in your project root module.  
 
### Parse Tree structure

It is possible to generate idiomatic Rust syntax trees. For this you would need to use labels feature of ANTLR tool.
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
`parser.build_parse_trees = false`, but unfortunately currently it will prevent visitors from working. 
  
### Differences with Java
Although Rust runtime API has been made as close as possible to Java, 
there are quite some differences because Rust is not an OOP language and is much more explicit. 

 - If you are using labeled alternatives, 
 struct generated for the rule is an enum with variant for each alternative
 - Parser needs to have ownership for listeners, but it is possible to get listener back via `ListenerId`
 otherwise `ParseTreeWalker` should be used.
 - In embedded actions to access parser you should use `recog` variable instead of `self`/`this`. 
 This is because predicates have to be inserted into two syntactically different places in generated parser 
 and in one of them it is impossible to have parser as `self`.
 - str based `InputStream` have different index behavior when there are unicode characters. 
 If you need exactly the same behavior, use `[u32]` based `InputStream`, or implement custom `CharStream`.
 - In actions you have to escape `'` in rust lifetimes with `\ ` because ANTLR considers them as strings, e.g. `Struct<\'lifetime>`
 - To make custom tokens you should use `@tokenfactory` custom action, instead of usual `TokenLabelType` parser option.
 ANTLR parser options can accept only single identifiers while Rust target needs know about lifetime as well. 
 Also in Rust target `TokenFactory` is the way to specify token type. As example you can see [CSV](grammars/CSV.g4) test grammar.
 - All rule context variables (rule argument or rule return) should implement `Default + Clone`.
 
### Unsafe
Currently, unsafe is used only for downcasting (through separate crate) 
and to update data inside Rc via `get_mut_unchecked`(returned mutable reference is used immediately and not stored anywhere)

### Versioning
In addition to usual Rust semantic versioning, 
patch version changes of the crate should not require updating of generator part 
  
## Licence

BSD 3-clause 
