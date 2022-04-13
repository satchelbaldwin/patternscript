# patternscript

a language for defining arcade bullet hell patterns in a clean and efficient way

#### quick look: syntax example

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

pattern p2 = {
     actions =  {
          spawn {

	     }
     }
}

pattern phase1 = {
     iteration_type = time;  // time instead of a cycle count
     length = 6.0 seconds;           // time in seconds
     actions = {
          origin = entity_position + (0, 20);

          for (n = 0...5) {
               angle = towards_player;
               spawn {
                    type = mid_sized;
                    position = origin;
                    direction = angle + n * 20;        // should handle precedence correctly here
                    speed = 200;
               }
               wait 4 frames;
          }
          wait 0.5 seconds;

          for (i = 0...3, j = 0...3) unless (i == 1 and j == 1) {
               x = 1;
               spawn {
                    type = mid_sized;
                    position_fn = downward_s_curve(t, 200, (entity_position + ((i - 1) * 80, (j - 1) * 80))); 
               }
          }
          wait 0.5 seconds;
     }
}
```

*(visualized example gif of what this would show at runtime yet to be implemented)*

## inspiration

(bulletml)[https://www.asahi-net.or.jp/~cs8k-cyu/bulletml/index_e.html], the go-to existing declarative way to write these patterns

visualizer ideas were inspired (by this implementation)[https://github.com/emillon/bulletml] of the above format

## features

* user defined functions to avoid repetition
* vector data types to handle position, colors in a dynamic and reasonable way
* adequate performance
* quick syntax for defining n-dimensional for loops over ranges
* time expressed by both frames/seconds
* child patterns and inheritance; composability

## implementation

the reference interpreter will hopefully be engine-agnostic and usable within other contexts than the demo i'm working on. 
at least, that is the current goal. 

included ~~is~~ will be a visualizer *(soon)* and a thin cli wrapper for showing the debug information on parsing.

## language documentation

broadly, the language is a list of top-level definitions.  
these definitions can be one of three things: a `pattern`, a `path`, or a `bullet.`  

`bullet`s are entities for use in `pattern`s; they are lists of declared variables.  
`path`s are user defined functions for x,y paths given time; this may be expanded into arbitrary user defined functions.  
`pattern`s are the meat of the definitions and contain all of the other behavior.  

the general syntax for top-level definitions is  
```
keyword name = {

}
```  
where path requires a list of arguments unlike the others; `path name(arg0, arg1, arg2...) = {}`.

code blocks between braces can either contain statements or imperative actions.  
declaration statements are in the form of  
```
variable = expression;
```  
where the built in type examples are  
```
var1 = 1.0;         // float
var2 = 1;           // int
var3 = "hello";     // string
var4 = (1, 2, 3)    // vector
var5 = var1 + var2; // int + float -> float
var6 = { x = 1; }   // block containing variable
```

imperative actions are in the form of keywords understood sequentially in a list.  
these are `for`, `wait`, and `spawn`.

for blocks iterate over ranges and can have conditionals.  
```
for (i = 0...3, j = 0...3) {
    
}
```  
would iterate 9 times, with the iterations 
* `i = 0`, `j = 0`
* `i = 0`, `j = 1`
* `i = 0`, `j = 2`
* `i = 1`, `j = 0`
* ...
* `i = 2`, `j = 2`

these blocks can also have conditional execution with the syntax  
```
for (i = 0...3, j = 0...3) unless (i == 1 and j == 1) {

}
```  
which would skip all cases where the condition on the right is true.  
`when` clauses mean that all cases where the block on the right is true are executed.  
`unless` clauses are the opposite.  

- more here later - 

## progress

* ☑ lexer
* ☑ parser
* runtime interpreter
* demo
* test cases
* documentation 