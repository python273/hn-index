#![feature(available_concurrency)]
use std::thread;
use std::env;
use std::io::Write;
use std::error;
use std::time::Instant;

use regex::Regex;


static FILE: &[u8] = include_bytes!("../hn-index.bin");


// const URL_START: &str = "https://news.ycombinator.com/item?id=";
const URL_START: &str = "https://hkrn.ws/";

/// Searches for the next FFFFFF marker
fn find_next_item(b: &[u8], i: usize) -> Option<usize> {
    let max = b.len() - 3;

    for i in i..max {
        if b[i] == 255 && b[i + 1] == 255 && b[i + 2] == 255 {
            return Some(i + 3);
        }
    }

    None
}

#[derive(Debug)]
struct Story<'a> {
    id: u32,
    comments: u32,
    title: &'a str
}


fn main() -> std::result::Result<(), Box<dyn error::Error>> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    /// Fixes piping
    /// https://github.com/rust-lang/rust/issues/46016
    macro_rules! println {
        () => (
            if let Err(_) = writeln!(handle) { return Ok(()); }
        );
        ($($arg:tt)*) => (
            if let Err(_) = writeln!(handle, $($arg)*) { return Ok(()); }
        );
    }

    let total_instant = Instant::now();

    let query = env::args().nth(1).expect("Gimme text");

    let min_comments = match env::args().nth(2) {
        Some(s) => s.parse::<u32>().unwrap(),
        _ => {0_u32}
    };

    // let file_instant = Instant::now();
    // let contents = fs::read("./titles.txt").expect("FILE");
    let contents = FILE;
    // println!("{:<16}{}", "Char count", contents.len());
    // let file_elapsed = file_instant.elapsed();

    let threads_num: usize = {
        thread::available_concurrency()
            .unwrap_or_else(|_| std::num::NonZeroUsize::new(1).unwrap())
            .get()
    };
    // let threads_num: usize = 1;
    let max_thread_id: usize = threads_num - 1;

    let chunk_size = contents.len() / threads_num;
    // println!("{:<16}{:?}", "avail threads", thread::available_concurrency());
    // println!("{:<16}{}", "threads_num", threads_num);
    // println!("{:<16}{}", "chunk_size", chunk_size);
    // println!("");

    let scan_instant = Instant::now();

    let mut stories = crossbeam::scope(|scope| {
        let mut last_end = 0;

        let handles: Vec<_> = (0..threads_num).into_iter().map(|x| {
            let start = last_end;
            let end = if x != max_thread_id { start + chunk_size } else { contents.len() };
            let end = find_next_item(&contents, end).unwrap_or(contents.len());

            last_end = end;

            let t_query = &query[..];
            let t_contents = &contents[start..end];
            let end_rel = end - start;

            scope.spawn(move |_| {
                let re: Regex = Regex::new(&t_query).unwrap();

                let mut stories: Vec<Story> = vec![];
                let mut i;
                let mut next_i: usize = 0;

                while next_i < end_rel {
                    i = next_i;

                    let title_len = t_contents[i] as usize;

                    next_i += 1 + 2 + title_len + 4 + 3;

                    let comments = u16::from_be_bytes([
                        t_contents[i + 1],
                        t_contents[i + 1 + 1]
                    ]) as u32;
                    if comments < min_comments { continue; }

                    let title_i_start = i + 3;
                    let title_i_end = i + 3 + title_len;

                    let title: &str = unsafe {
                        std::str::from_utf8_unchecked(&t_contents[title_i_start..title_i_end])
                    };
                    if !re.is_match(&title) { continue; }

                    let id = u32::from_be_bytes([
                        t_contents[title_i_end],
                        t_contents[title_i_end + 1],
                        t_contents[title_i_end + 2],
                        t_contents[title_i_end + 3],
                    ]);

                    stories.push(Story { id, comments, title });
                };

                stories
            })
        }).collect();

        let mut stories: Vec<Story> = vec![];

        for handle in handles {
            stories.append(&mut handle.join().unwrap())
        }

        stories
    }).unwrap();

    stories.sort_by_key(|s| u32::MAX - s.id);

    let scan_elapsed = scan_instant.elapsed();

    let stories_len = stories.len();

    let print_instant = Instant::now();

    let re: Regex = Regex::new(&query).unwrap();

    for s in stories {
        // TODO: check term supports ansi colors / isatty
        // highlight with ansi colors
        let title = re.replace_all(
            s.title,
            "\x1b[1m\x1b[31m${0}\x1b[0m"
        );
        // account for ansi codes in padding
        let w = 80 + (title.len() - s.title.len());

        if s.comments > 0 {
            println!("{:>4} {:<width$} {}{}", s.comments, title, URL_START, s.id, width = w);
        } else {
            println!("     {:<width$} {}{}", title, URL_START, s.id, width = w);
        }
        
    }
    let print_elapsed = print_instant.elapsed();

    println!();
    println!("{:<14}{}", "Found stories", stories_len);
    // println!("{:<14}{:?}", "Read time", file_elapsed);
    println!("{:<14}{:?}", "Scan time", scan_elapsed);
    println!("{:<14}{:?}", "Print time", print_elapsed);
    println!("{:<14}{:?}", "Total time", total_instant.elapsed());

    Ok(())
}
