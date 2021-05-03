#![feature(available_concurrency)]
use std::thread;
use std::env;
// use std::fs;
use std::time::Instant;

use regex::Regex;
// use simdutf8::basic::from_utf8;


static FILE: &str = include_str!("../titles.txt");


// const URL_START: &str = "https://news.ycombinator.com/item?id=";
const URL_START: &str = "https://hkrn.ws/";


fn find_utf8_char_end(s: &str, i: usize) -> usize {
    let mut end = i;
    while !s.is_char_boundary(end) {
        end += 1;
    }
    end
}


#[derive(Debug)]
struct Story<'a> {
    id: u32,
    comments: u32,
    title: &'a str
}

fn main() {
    let total_instant = Instant::now();

    let query = env::args().nth(1).expect("Gimme text");

    let min_comments = match env::args().nth(2) {
        Some(s) => s.parse::<u32>().unwrap(),
        _ => {0_u32}
    };

    // let file_instant = Instant::now();
    // let contents = &fs::read_to_string("./titles.txt").expect("FILE")[..];
    // let contents = fs::read("./titles.txt").expect("FILE");
    // let contents = from_utf8(&contents).unwrap();
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
            let end = find_utf8_char_end(&contents, end);
            let end = contents[..end].rfind('\n').unwrap();

            last_end = end + 1;

            let t_query = &query[..];
            let t_contents = &contents[start..end];

            scope.spawn(move |_| {
                let re: Regex = Regex::new(&t_query).unwrap();

                t_contents
                    .split('\n')
                    .filter(|&s| u32::from_str_radix(&s[8..12], 16).unwrap() >= min_comments)
                    .filter(|&s| re.is_match(&s[12..]))
                    .map(|s| {
                        let id = u32::from_str_radix(&s[0..8], 16).unwrap();
                        let comments = u32::from_str_radix(&s[8..12], 16).unwrap();
                        let title = &s[12..];

                        Story { id, comments, title }
                    })
                    .collect()
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
    for s in stories {
        if s.comments > 0 {
            println!("{:>4} {:<80} {}{}", s.comments, s.title, URL_START, s.id);
        } else {
            println!("     {:<80} {}{}", s.title, URL_START, s.id);
        }
        
    }
    let print_elapsed = print_instant.elapsed();

    println!();
    println!("{:<14}{}", "Found stories", stories_len);
    // println!("{:<14}{:?}", "Read time", file_elapsed);
    println!("{:<14}{:?}", "Scan time", scan_elapsed);
    println!("{:<14}{:?}", "Print time", print_elapsed);
    println!("{:<14}{:?}", "Total time", total_instant.elapsed());
}
