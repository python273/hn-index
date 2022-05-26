# hn-index

Search Hacker News stories' titles with [regex](https://docs.rs/regex/1.5.6/regex/index.html#syntax).

```
$ wget https://python273.keybase.pub/hackernews-stories-dump-994369_31514311.tar.gz
$ tar xf hackernews-stories-dump-994369_31514311.tar.gz
$ cargo build --release
$ ./target/release/hn-index "\b[Rr]ust"
[...]
 101 Rust Language Cheat Sheet                                                        https://hkrn.ws/26930908
  45 Show HN: High-speed UTF-8 validation in Rust                                     https://hkrn.ws/26887438
[...]

Found stories 8923
Scan time     16.251403ms
Print time    27.237503ms
Total time    43.549156ms
```

`hn-index (regex) [min comments]`

![](./term.png)

## Dump

Source: https://hn.algolia.com/api

Header: `[checkpoint; u32 little endian]*31` (offsets to start scanning by CPU threads)

Format: `[title len; u8][comments num; u16 little endian][title; utf-8][id; u32 little endian]`

## Bonus memes

![Title length](./hn-titles-plt.png)
![Comments](./hn-comments-plt.png)
![Points](./hn-points-plt.png)
