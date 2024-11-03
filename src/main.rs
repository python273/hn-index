use std::convert::TryInto;
use std::thread;
use std::env;
use std::io::Write;
use std::error;
use std::fs;
use std::time::Instant;
use std::cmp;

use regex::Regex;

#[derive(Debug)]
enum SortBy {
    StoryId,
    Comments
}

const CHECKPOINTS_COUNT: usize = 31;

static FILE: &[u8] = include_bytes!("../hn-index.bin");

// const URL_START: &str = "https://news.ycombinator.com/item?id=";
// const URL_START: &str = "https://hkrn.ws/";
const URL_START: &str = "https://hn.cns.wtf/#";

#[derive(Debug)]
struct Story<'a> {
    id: u32,
    comments: u16,
    title: &'a str
}


fn main() -> std::result::Result<(), Box<dyn error::Error>> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    /// Fixes piping error
    /// https://github.com/rust-lang/rust/issues/46016
    macro_rules! xprintln {
        () => (
            if let Err(_) = writeln!(handle) { return Ok(()); }
        );
        ($($arg:tt)*) => (
            if let Err(_) = writeln!(handle, $($arg)*) { return Ok(()); }
        );
    }

    let total_instant = Instant::now();

    let mut sort_by = SortBy::StoryId;
    let mut pos_arg_i: usize = 0;

    for i in env::args().skip(1) {
        if !i.starts_with("--") { break; }
        pos_arg_i += 1;

        match &i[..] {
            "--comments" => {
                sort_by = SortBy::Comments;
            },
            &_ => {
                break
            }
        }
    }

    let query = env::args().nth(pos_arg_i + 1).expect("Gimme regex");

    let min_comments = match env::args().nth(pos_arg_i + 2) {
        Some(s) => s.parse::<u16>().unwrap(),
        _ => {0_u16}
    };

    // let file_instant = Instant::now();
    // let contents = fs::read("hn-index.bin").expect("FILE");
    let contents = FILE;
    // xprintln!("{:<16}{}", "Char count", contents.len());
    // let file_elapsed = file_instant.elapsed();

    let threads_num: usize = {
        thread::available_parallelism()
            .unwrap_or_else(|_| std::num::NonZeroUsize::new(1).unwrap())
            .get()
    };
    let threads_num = cmp::min(CHECKPOINTS_COUNT + 1, threads_num);
    // let threads_num: usize = 1;
    let max_thread_id: usize = threads_num - 1;

    // xprintln!("{:<16}{:?}", "avail threads", thread::available_parallelism());
    // xprintln!("{:<16}{}", "threads_num", threads_num);

    let mut checkpoints: [usize; CHECKPOINTS_COUNT] = [0; CHECKPOINTS_COUNT];
    for (i, item) in checkpoints.iter_mut().enumerate() {
        let a = i * 4;
        *item = u32::from_le_bytes(contents[a..a+4].try_into().unwrap()) as usize;
    }

    let skip_per_thread = (CHECKPOINTS_COUNT + 1) / threads_num;

    let scan_instant = Instant::now();

    let query_re: Regex = Regex::new(&query).unwrap();

    let mut stories: Vec<Story> = {
        let mut last_end = CHECKPOINTS_COUNT * 4;  // skipping checkpoints data

        let handles: Vec<_> = (0..threads_num).map(|x| {
            let start = last_end;
            let end = if x != max_thread_id {
                checkpoints[(x + 1) * skip_per_thread - 1]
            } else { contents.len() };

            last_end = end;

            let re = query_re.clone();
            let t_contents = &contents[start..end];
            let end_rel = end - start;

            thread::spawn(move || {
                let mut matched_stories: Vec<Story> = vec![];
                let mut i;
                let mut next_i: usize = 0;

                while next_i < end_rel {
                    i = next_i;

                    let title_len = t_contents[i] as usize;

                    next_i += 1 + 2 + title_len + 4;

                    let comments = u16::from_le_bytes(
                        t_contents[i+1..i+3].try_into().unwrap());
                    if comments < min_comments { continue; }

                    let title_i_start = i + 1 + 2;
                    let title_i_end = title_i_start + title_len;

                    let title: &str = unsafe {
                        std::str::from_utf8_unchecked(&t_contents[title_i_start..title_i_end])
                    };
                    if !re.is_match(title) { continue; }

                    let id = u32::from_le_bytes(
                        t_contents[title_i_end..title_i_end+4].try_into().unwrap());

                    matched_stories.push(Story { id, comments, title });
                };

                matched_stories
            })
        }).collect();

        handles.into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    };



    match sort_by {
        SortBy::StoryId => {
            stories.sort_by_key(|s| u32::MAX - s.id);
        },
        SortBy::Comments => {
            stories.sort_by_key(|s| u16::MAX - s.comments);
        }
    }

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
            xprintln!("{:>4} {:<width$} {}{}", s.comments, title, URL_START, s.id, width = w);
        } else {
            xprintln!("     {:<width$} {}{}", title, URL_START, s.id, width = w);
        }
        
    }
    let print_elapsed = print_instant.elapsed();

    xprintln!();
    xprintln!("{:<14}{}", "Found stories", stories_len);
    // xprintln!("{:<14}{:?}", "Read time", file_elapsed);
    xprintln!("{:<14}{:?}", "Scan time", scan_elapsed);
    xprintln!("{:<14}{:?}", "Print time", print_elapsed);
    xprintln!("{:<14}{:?}", "Total time", total_instant.elapsed());

    Ok(())
}
