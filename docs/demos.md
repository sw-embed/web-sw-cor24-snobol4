# SNOBOL4 Demos

This page documents the embedded demo programs included in the web UI.

## array

Demonstrates array creation and indexed access.

```sno
        NAMES = ARRAY('1:50')
        NAMES<1> = 'Alice'
        NAMES<2> = 'Bob'
        NAMES<3> = 'Carol'
        OUTPUT = NAMES<1>
        OUTPUT = NAMES<2>
        OUTPUT = NAMES<3>
END
```

## break

Parses a pipe-delimited record using `BREAK` pattern matching.

```sno
        LINE = 'Alice|F|English'
        LINE BREAK('|') . NAME '|' BREAK('|') . SEX '|' REM . MAJOR :F(NO)
        OUTPUT = NAME
        OUTPUT = SEX
        OUTPUT = MAJOR :(END)
NO     OUTPUT = 'parse failed'
END
```

## concat

String concatenation with implicit juxtaposition.

```sno
        NAME = 'World'
        OUTPUT = 'Hello, ' NAME '!'
END
```

## count

Simple counter loop with arithmetic.

```sno
        N = 1
LOOP   OUTPUT = N
        N = N + 1
        OUTPUT = N
        OUTPUT = N + 1
END
```

## fizzbuzz

Classic FizzBuzz using `REMDR` (remainder) and conditional branches.

```sno
* FizzBuzz 1..15 in SNOBOL4
 N = 1
L R = REMDR(N,15)
 EQ(R,0) :S(A)
 R = REMDR(N,3)
 EQ(R,0) :S(B)
 R = REMDR(N,5)
 EQ(R,0) :S(C)
 OUTPUT = N :(I)
A OUTPUT = 'FizzBuzz' :(I)
B OUTPUT = 'Fizz' :(I)
C OUTPUT = 'Buzz'
I N = N + 1
 LE(N,15) :S(L)
END
```

## hello

Minimal "Hello, World!" program.

```sno
        OUTPUT = 'Hello, World!'
END
```

## hello-goto

Demonstrates explicit goto to skip code (unreachable line).

```sno
        OUTPUT = 'Hello, World!' :(END)
        OUTPUT = 'This should not print'
END
```

## input

Reads lines from input data and echoes them to output.
Requires input data: `Hello`, `World`, `Goodbye`.

```sno
READ   LINE = INPUT                :F(DONE)
        OUTPUT = LINE               :(READ)
DONE   OUTPUT = 'End of input'
END
```

## multiply

Arithmetic multiplication.

```sno
        X = 7
        Y = X * 6
        OUTPUT = Y
END
```

## span

Matches leading digits using `SPAN` pattern.

```sno
        DIGITS = SPAN('0123456789')
        TEXT = 'abc 123 xyz'
        TEXT DIGITS . N :F(NO)
        OUTPUT = N :(END)
NO     OUTPUT = 'no match'
END
```

## span-fail

Demonstrates pattern failure when `SPAN` finds no leading digits.

```sno
        DIGITS = SPAN('0123456789')
        TEXT = 'abc xyz'
        TEXT DIGITS . N :F(NO)
        OUTPUT = N :(END)
NO     OUTPUT = 'no match'
END
```
