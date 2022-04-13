# patternscript_cli

thin wrapper to print parse trees 

## building

requirements: a working rust/cargo installation

`cargo build`

## usage

```
./patternscript [action] [file]
    actions:
        -p : parse
        -l : lex
    file: 
        a patternscript file, see examples
```

example:

`cargo run -- -p example/1.pattern`
