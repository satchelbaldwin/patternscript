# ps

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

## prior work

the venerable (bulletml)[https://www.asahi-net.or.jp/~cs8k-cyu/bulletml/index_e.html]
visualizer ideas and reference patterns to compare to (from this implementation)[https://github.com/emillon/bulletml]

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
      | id '=' args; // tuple
      | wait time;
      | for_block
      | spawn block
```


example: 

```
bullet mid_sized = {
     sprite = "gameasset";
     hitbox = (4, 4);
     color = (255, 255, 0);
}

path downward_s_curve(t, speed, offset) = {
     x = (50 * sin(t)) + offset;
     y = t * speed;
}

pattern phase1 = {
     iteration_type = time;  // time instead of a cycle count
     length = 6.0 seconds;           // time in seconds
     actions = {
          origin = entity_position + (0, 20);

          for (n = 0...5) {
               angle = towards_player;
               spawn {
                    bullet = mid_sized,
                    position = origin,
                    direction = angle + n * 20,        // should handle precedence correctly here
                    path = simple,
                    speed = 200,
               }
               wait 4 frames;
          }
          wait 0.5 seconds;

          for (i = 0...3, j = 0...3) unless (i == 1 and j == 1) {
               spawn {
                    bullet = mid_sized,
                    path = downward_s_curve(t, 200, (entity_position + ((i - 1) * 80, (j - 1) * 80))), 
               }
          }
          wait 0.5 seconds;
     }
}
```