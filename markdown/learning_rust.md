## Learning Rust ##

### Do we need another language? ###

So, it's been some time since I've seen a language that really delighted me. Full disclosure, I enjoyed doing some <a href="https://wiki.gnome.org/Projects/Vala" target="_blank">Vala</a> back in the 2011. The language, which is actually just a wrapper around C + GObject, had some fresh ideas for those of us who wanted to do Systems Programming but also aspired to something more modern than C or C++.

## Some tricks ##

### Sending traits over channels ###
As usual, Stackoverflow is an excellent resource. I picked up this little gem about sending traits over a task boundary <a href="http://stackoverflow.com/questions/25649423/sending-trait-objects-between-tasks-in-rust" target="_blank">here</a>. It was really helpful to get a better grasp of the use about <a href="https://github.com/rust-lang/rust/blob/master/src/liballoc/boxed.rs#L53" target="_blank">Box</a> and the <a href="https://github.com/rust-lang/rust/blob/master/src/libcore/marker.rs#L33" target="_blank">Send</a> trait.

### Struct variants ###
<a href="https://github.com/rust-lang/rfcs/blob/master/text/0418-struct-variants.md" target="_blank">Struct variants</a> in enum are very convenient. Compare this variant which uses a tuple:

```rust
enum FrontendMessage<'a> {
    // ...
    Bind(&'a str, &'a str, &'a [i16], &'a [Option<Vec<u8>>], &'a [i16]),
    // ...
}
```

with this, almost self-evident, struct variant version:


```rust
enum FrontendMessage<'a> {
    // ...
    Bind {
        pub portal: &'a str,
        pub statement: &'a str,
        pub formats: &'a [i16],
        pub values: &'a [Option<Vec<u8>>],
        pub result_formats: &'a [i16]
    },
    // ...
}
```
