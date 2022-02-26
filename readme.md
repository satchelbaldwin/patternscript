# ps

```
comment := // until EOL (\n) (lexer ignores, not part of grammar)

id := [a-zA-Z]+[a-zA-Z0-9]*
num := int | float
int := [0-9]*
float := [0-9]*.([0-9]*)?

block := stmt | { stmts } 

stmts := ε | stmt stmts

stmt := ex
      | pattern id 
```


example: 

```
pattern phase1 {
     iteration_type = time;
     length = 6.0; // time in seconds
     actions {
          do (n: 5) times unless (n = 3) {
               
          }
          delay 0.5;
          do (i: 3, j: 3) times unless (i = 1 or j = 1) {

          }
          delay 0.5;
     }
}

pattern phase2 {
     iteration_type count;

}