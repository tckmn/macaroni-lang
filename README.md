# macaroni-lang

An esoteric programming language designed to contain as few instructions as
possible.

## Types

Macaroni has two types:

- Numbers, which are represented as 64-bit floating point internally.

- Arrays, which can contain other arrays or numbers.

However, it also has:

- "Strings," which are just arrays of numbers. The only place that Macaroni
  distinguishes between "strings" and arrays is in the I/O operators (`print`
  and `read`) and base operators (`tobase` and `frombase`).

- Some operators accept labels and expect them to set the `_` variable before
  returning. These are used as a primitive form of "blocks" or "subroutines."

## Operators

Macaroni has all of 20 operators.

### Number operators

- `nn` -> `n`: add
- `nn` -> `n`: multiply
- `n` -> `n`: floor
- `nn` -> `n`: power
- `nn` -> `a`: tobase (\*returns an array-"string")

### Array operators

- `al` -> `a`: sort
- `aa` -> `a`: cat (concatenate)
- `an` -> `a`: each (a la Ruby `each_cons`, `each_slice` if arg. negative)
- `al` -> `a`: map
- `al` -> `a`: index (indeces)
- `annn` -> `a`: slice (a la Python `a[b:c:d]`)
- `a` -> `n`: length
- `a` -> `a`: transpose
- `an` -> `a`: flatten (0 = completely flatten)
- `an` -> `n`: frombase (\*for array-"strings" only)

### Other operators

- `*` -> `a`: wrap (in array)
- `a` -> `-`: print
- `-` -> `a`: read (a single line, from STDIN)
- `-` -> `n`: rand (`[0,1)`)
- `-` -> `n`: time

## Syntax

The syntax of Macaroni is similar to many other languages, in that operators are called in *prefix notation*. In prefix notation, the name of the operator comes first, then the arguments. For example, `func(arg1, arg2)`.

However, the way Macaroni calls operators is slightly more unique. Each operator has a defined *arity*, which is simply the number of arguments it expects. This eliminates the need for parentheses. So, instead of saying `add(1, mult(2, 3))`, you would just say `add 1 mult 2 3`.

Since each operator has its own arity, there is no ambiguity with this syntax, and a simple recursive parser can be used.

## Control flow

Macaroni has two methods of control flow: label and goto.

- Labels are denoted by `/labelname`. They can exist anywhere before or after a
  complete statement.

- Gotos are `\labelname`. They are allowed in the same places as labels are.
  Additionally, a goto without a label name (simply ``\``) will act as a
  "return" and jump back to the last place a goto was called from.

## Variables

The syntax for setting variables is the following:

    :a 5

This would set the variable `a` to `5`. You can set a variable to any full
expression:

    :a add multiply 3 pow 2 -1 2

`a` would now be set to `3.5`.

The default value of all variables is `0`.

## Common operations / combinations

Since Macaroni only has 20 operators, naturally there are tons of them that
have been left out. Here are some implementations of common functions that you
would expect to find in other languages.

- Absolute value of `x`

        map slice " " 0 x 1 a
        /a set x multiply x -1 \

    A cleverer version that squares and then square roots the number:

        pow pow x 2 pow 2 -1

- `x` modulo `y` (using [a formula found on Wikipedia](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions#Mod_operator))

        add x multiply -1 multiply y floor multiply x pow y -1

- A "while loop" ("calls" `func` while `x` is equal to `0`)

        /WhileLoop
            map
                map slice " " x add x 1 1 func
                WhileLoop

        /func <stuff> \

## Why "macaroni"?

Macaroni's control flow is done entirely through `goto`s. The name "Macaroni"
is simply a spin-off of the aptly-named ["spaghetti
code."](https://en.wikipedia.org/wiki/Spaghetti_code)

## Contributing / credits

If you would like to contribute to Macaroni or suggest ideas, feel free to
either open an issue / pull request here on Github or ping me `@Doorknob` in
[The Nineteenth
Byte](http://chat.stackexchange.com/rooms/240/the-nineteenth-byte), the Stack
Exchange chatroom for [PPCG](http://codegolf.stackexchange.com).

*Huge* thanks to [@BlacklightShining](https://github.com/BlacklightShining) for
tons of help with implementing the language and spending their time to save
waaaay more of mine (while teaching me lots and lots about how Rust works in
the process). :)

Also, thanks to the `#rust` IRC channel for being awesome as always.
