# usd2nis

Small utility to retrieve USD to NIS/ILS exchange rate.

## Installing

```
$ cargo install --git https://github.com/guyru/usd2nis
```

Minimum supported Rust version: 1.65.

## Usage

```
USAGE:
    usd2nis <date> [USD]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <date>      Conversion date
    <USD>...    USD amounts to convert
```

The following will convert $15 to NIS using 20/12/2021 exchange rate.

```
$ usd2nis 2021-12-20 15
```

## Authors

* Author: [Guy Rutenberg](https://www.guyrutenberg.com/)
