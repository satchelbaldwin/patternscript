["target/debug/patternscript", "examples/1.pattern"]
Keyword(Bullet)
Id("mid_sized")
Assign
OpenBlock
Id("sprite")
Assign
String("gameasset")
Semicolon
Id("hitbox")
Assign
OpenParen
Number("4")
Comma
Number("4")
CloseParen
Semicolon
Id("color")
Assign
OpenParen
Number("255")
Comma
Number("255")
Comma
Number("0")
CloseParen
Semicolon
CloseBlock
Keyword(Path)
Id("downward_s_curve")
OpenParen
Id("t")
Comma
Id("speed")
Comma
Id("offset")
CloseParen
Assign
OpenBlock
Id("x")
Assign
OpenParen
Number("50")
Operator(Mul)
Id("sin")
OpenParen
Id("t")
CloseParen
CloseParen
Operator(Add)
Id("offset")
Semicolon
Id("y")
Assign
Id("t")
Operator(Mul)
Id("speed")
Semicolon
CloseBlock
Keyword(Pattern)
Id("phase1")
Assign
OpenBlock
Id("iteration_type")
Assign
Id("time")
Semicolon
Comment
Id("length")
Assign
Number("6.0")
Keyword(Seconds)
Semicolon
Comment
Id("actions")
Assign
OpenBlock
Keyword(Let)
Operator(Or)
Id("rigin")
Assign
Id("entity_position")
Operator(Add)
OpenParen
Number("0")
Comma
Number("20")
CloseParen
Semicolon
Keyword(For)
OpenParen
Id("n")
Assign
Number("0")
RangeSeparator
Number("5")
CloseParen
OpenBlock
Keyword(Let)
Id("angle")
Assign
Id("towards_player")
Semicolon
Keyword(Spawn)
OpenBlock
Id("type")
Assign
Id("mid_sized")
Comma
Id("position")
Assign
Operator(Or)
Id("rigin")
Comma
Id("direction")
Assign
Id("angle")
Operator(Add)
Id("n")
Operator(Mul)
Number("20")
Comma
Comment
Id("speed")
Assign
Number("200")
Comma
CloseBlock
Keyword(Wait)
Number("4")
Keyword(Frames)
Semicolon
CloseBlock
Keyword(Wait)
Number("0.5")
Keyword(Seconds)
Semicolon
Keyword(For)
OpenParen
Id("i")
Assign
Number("0")
RangeSeparator
Number("3")
Comma
Id("j")
Assign
Number("0")
RangeSeparator
Number("3")
CloseParen
Condition(Unless)
OpenParen
Id("i")
Operator(Test)
Number("1")
Operator(And)
Id("j")
Operator(Test)
Number("1")
CloseParen
OpenBlock
Keyword(Spawn)
OpenBlock
Id("type")
Assign
Id("mid_sized")
Comma
Id("position_fn")
Assign
Id("downward_s_curve")
OpenParen
Id("t")
Comma
Number("200")
Comma
OpenParen
Id("entity_position")
Operator(Add)
OpenParen
OpenParen
Id("i")
Operator(Sub)
Number("1")
CloseParen
Operator(Mul)
Number("80")
Comma
OpenParen
Id("j")
Operator(Sub)
Number("1")
CloseParen
Operator(Mul)
Number("80")
CloseParen
CloseParen
CloseParen
Comma
CloseBlock
CloseBlock
Keyword(Wait)
Number("0.5")
Keyword(Seconds)
Semicolon
CloseBlock
CloseBlock
Keyword(Pattern)
Semicolon
Keyword(Pattern)
Semicolon
EOF