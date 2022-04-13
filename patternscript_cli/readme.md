# patternscript_cli

thin wrapper to print parse trees / lex info

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
`patternscript_cli -p example/1.pattern`