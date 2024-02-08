# lox-r

The book Crafting Interpreter (https://www.craftinginterpreters.com/) shows how to implement a language interpreter and a virtual machine. This repo contains the the lox language interpreter written in Rust. 

The implementation was sometimes quite challenging, because the author of the book used Java and Rust is very strict when it comes to references. However, I learned a lot about Rust, programming languages, and reference handling in general.

In addition to the `clock` function, there are two extra native functions:
- `input`: Read user input from console.
- `readFile`: Read file content from disk.

The repo also contains some lox example code:
- `./examples/list.lox`: A linked list implementation.
- `./examples/native.lox`: Demonstrates the additional native functions.
- and more...
