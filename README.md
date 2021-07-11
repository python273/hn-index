# hn-index

Search Hacker News stories' titles with [regex](https://docs.rs/regex/1.5.4/regex/index.html#syntax).

```
$ wget https://python273.keybase.pub/hackernews-stories-dump-994369_27800535.tar.gz
$ tar xf hackernews-stories-dump-994369_27800535.tar.gz
$ cargo +nightly build --release
$ ./target/release/hn-index "\b[Rr]ust"
[...]
 101 Rust Language Cheat Sheet                                                        https://hkrn.ws/26930908
  45 Show HN: High-speed UTF-8 validation in Rust                                     https://hkrn.ws/26887438
[...]

Found stories 8009
Scan time     15.228174ms
Print time    25.479477ms
Total time    40.747675ms
```

`hn-index (regex) [min comments]`

![](./term.png)

## Dump

Source: https://hn.algolia.com/api

Header: `[checkpoint; u32]*16` (used to find places to start scanning for each thread)
Format: `[title len; u8][comments num; u16 big endian][title; utf-8][id; u32 big endian]`

## Bonus memes

![Title length](./hn-titles-plt.png)
![Comments](./hn-comments-plt.png)
![Points](./hn-points-plt.png)
