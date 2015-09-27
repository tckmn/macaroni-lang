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

Macaroni has all of 21 operators.

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
- `v*` -> `-`: set

## Control flow

Macaroni has two methods of control flow: label and goto.

- Labels are denoted by `/labelname`. They can exist anywhere before or after a
  complete statement.

- Gotos are `\labelname`. They are allowed in the same places as labels are.
  Additionally, a goto without a label name (simply ``\``) will act as a
  "return" and jump back to the last place a goto was called from.

## Why "macaroni"?

Macaroni's control flow is done entirely through `goto`s. The name "Macaroni"
is simply a spin-off of the aptly-named ["spaghetti
code."](https://en.wikipedia.org/wiki/Spaghetti_code).

## Credits

*Huge* thanks to [@BlacklightShining](https://github.com/BlacklightShining) for
tons of help with implementing the language and spending their time to save
waaaay more of mine (while teaching me lots and lots about how Rust works in
the process). :)

Also, thanks to the `#rust` IRC channel for being awesome as always.
