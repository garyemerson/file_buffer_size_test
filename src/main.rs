use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::{env, io};
use std::time::{/*self,*/ SystemTime};
use std::num::Wrapping;
use rand::Rng;
// use std::thread;


fn main() {
    let path = env::args().nth(1).expect("Must provide file path.");
    println!("path: {}", path);
    let mut f = File::open(&path).expect(&format!("Failed to open file {}", path));
    let mut buffer = [0; 4_194_304];
    let mut checksum: Wrapping<u8> = Wrapping(0);
    let mut i = 0;
    let mut sum_read: u64 = 0;
    let mut times: HashMap<usize, (u128, u64)> = HashMap::new();
    let now = SystemTime::now();
    let mut printer = Printer::new();
    loop {
        let exp: u32 = rand::thread_rng().gen_range(10, 23);
        let len = 1 << exp;
        let buffer_tmp = &mut buffer[..len];
        let start = std::time::Instant::now();
        let num_read = f.read(buffer_tmp).expect("read call failed");
        let elapsed_nano = start.elapsed().as_nanos();
        let entry = times.entry(len).or_insert((0, 0));
        entry.0 += elapsed_nano;
        entry.1 += num_read as u64;
        sum_read += num_read as u64;
        if num_read == 0 {
            println!("\nFinished reading file");
            break;
        }
        if i % 10 == 0 {
            printer.print(
                format!(
                    "{} read {} bytes\n{} total ({}/s)\nstats:\n{}",
                    i,
                    num_read,
                    bytes_to_human(sum_read),
                    bytes_to_human((sum_read as f64 / now.elapsed().expect("elapsed").as_secs() as f64) as u64),
                    calc_stats(&times)));
        }
        if num_read == buffer_tmp.len() {
            checksum += buffer_tmp.iter().map(|x| Wrapping(*x)).last().unwrap(); //.sum::<Wrapping<u8>>();
        } else if num_read < buffer_tmp.len() {
            checksum += buffer_tmp.iter().take(num_read).map(|x| Wrapping(*x)).last().unwrap(); //.sum::<Wrapping<u8>>();
        } else {
            panic!("num_read is greater than buffer length, the world is ending!");
        }
        i += 1;
    }
    println!("checksum is {}", checksum);
}

fn calc_stats(times: &HashMap<usize, (u128, u64)>) -> String {
    let mut rates = times.iter()
        .map(|(size, (nanos, total_read))| {
            let bytes_per_sec = *total_read as f64 * 1e9 / *nanos as f64;
            (*size, bytes_per_sec)
        })
        .collect::<Vec<(usize, f64)>>();
    rates.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("partial_cmp"));
    rates.into_iter()
        .take(7)
        .map(|(size, rate)| format!("{}: {}/s", size, bytes_to_human(rate as u64)))
        .collect::<Vec<String>>()
        .join("\n")
}

fn bytes_to_human(num_bytes: u64) -> String {
    let prefixes = [("GB", 1e9), ("MB", 1e6), ("KB", 1e3)];
    for (p, n) in &prefixes {
        if num_bytes as f64 >= *n {
            return format!("{:.2}{}", num_bytes as f64/ n, p);
        }
    }
    format!("{}B", num_bytes)
}

struct Printer {
    lines_printed: Option<i32>
}

impl Printer {
    fn new() -> Printer {
        Printer { lines_printed: None }
    }

    fn print(&mut self, s: String) {
        let clear_line_str = "\r\x1b[K";
        let move_up_str = "\x1b[1A";
        let num_clears = self.lines_printed.unwrap_or(0) as usize + 1;
        let reset_str: String = vec![clear_line_str; num_clears].join(move_up_str);

        print!("{}{}", reset_str, s);
        io::stdout().flush().ok().expect("Could not flush stdout");

        let num_nls = s.chars().filter(|c| *c == '\n').count() as i32;
        self.lines_printed = Some(num_nls);
    }
}
