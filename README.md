# hn-index

```
$ wget https://python273.keybase.pub/hackernews-stories-dump-994369_27045741.tar.gz
$ tar xf hackernews-stories-dump-994369_27045741.tar.gz
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

![](./term.png)

## Dump

Source: https://hn.algolia.com/api

Format: `[title len; u8][comments num; u16 big endian][title; utf-8][id; u32 big endian]\xFF\xFF\xFF`

## Bonus memes

![Title length](./hn-titles-plt.png)
![Comments](./hn-comments-plt.png)
![Points](./hn-points-plt.png)
