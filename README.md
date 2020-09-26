# Truth Table Latex Generator

A simple program to generate the latex for a rust program. Most things are
non-configurable. Just wrote this for myself. Use it if you are happy with the
style

## Using

To run the program use:
```
cargo run
```
If you do not want to print the steps use
```
cargo run -- --no-steps
```

This will bring up a prompt where you can type in the expression you want
resolved. Type the expression with all explicit parentheses as there is no operator
precedence or direction. The following syntaxes are available and can be
composed to arbitrary depth.

```
A LogicalExp is one of:
 - Variable       : any symbol
 - Not            : (- LogicExp)
 - And            : (LogicExp * LogicExp) 
 - Or             : (LogicExp + LogicExp) 
 - Implies        : (LogicExp => LogicExp) 
 - If and Only If : (LogicExp <=> LogicExp) 
```

The program will then print the latex table and bring up the prompt again

