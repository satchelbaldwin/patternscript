# patternscript

the patternscript language: parser and interpreter  
this is exclusively the library for working with the language and generating/manipulating the AST  
this is an educational project meant to use minimal dependencies, so the wheel will be reinvented poorly here

## building

requirements: a working rust/cargo installation

`cargo build`

## the language

```
comment := // until EOL (\n) (lexer ignores, not part of grammar)

id := [a-zA-Z]+[a-zA-Z0-9]*     // lvalues
num := int 
     | float              
int := [0-9]*
float := [0-9]*.([0-9]*)?

rvalue := num 
        | string 
        | '"' [a-zA-Z0-9]* '"' 

expression = rvalue | expr | function_call | time
time := int frames | float seconds

test := test == bool 
      | bool
bool := bool and expr
      | bool or expr
      | expr
expr := expr + term 
      | expr - term 
      | term
term := term * factor 
      | term / factor 
      | factor
factor := exp ^ factor 
        | exp
exp := rvalue 
     | '(' test ')'

block := stmt ; 
       | '{' stmt { ';' stmt } '}' 

args := expression { , expression }
argdef := id { , id }

function_call := id '(' args ')'

range := int '...' int 

for_decl := id = range { , id = range }

cond := unless | when

for_block := for '(' for_decl ')' [ cond '(' expr ')' ]  block

stmt := pattern id '=' block
      | bullet id '=' block 
      | path id '(' argdef ')' '=' block
      | id '=' expression;
      | id '=' args; 
      | wait time;
      | for_block
      | spawn block
```

## notes

[excellent expression parsing article](https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm), all linked in the bibliography are good as well