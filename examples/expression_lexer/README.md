# Expression Lexer

A simple example that converts numbers and operators into a token stream with location information.

```sh
$ cargo run "2 + 4   -  10"
> [
>   Spanned {
>     start: 0,
>     token: Num,
>     end: 1,
>   },
>   Spanned {
>     start: 2,
>     token: Op,
>     end: 3,
>   },
>   Spanned {
>     start: 4,
>     token: Num,
>     end: 5,
>   },
>   Spanned {
>     start: 8,
>     token: Op,
>     end: 9,
>   },
>   Spanned {
>     start: 11,
>     token: Num,
>     end: 13,
>   },
> ]
```


