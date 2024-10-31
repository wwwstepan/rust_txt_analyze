use std::env;
use std::path::Path;
use std::fs::{self, File};
use std::io::{self, Read, BufRead, BufReader};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use rand::Rng;
use encoding_rs::WINDOWS_1251;
use rustring_builder::StringBuilder;
use chrono::Local;
//use rayon::prelude::*;

fn main() -> io::Result<()> {
	const ERR_ARGS_MESSAGE: &str = "Usage: txt_analize <file_name> [<num_words> = 200]";

	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		println!("{ERR_ARGS_MESSAGE}");
		return Ok(());
	}

	let path_to_file_or_dir = &args[1];
	let arg_path = Path::new(path_to_file_or_dir);
	
	if !arg_path.exists() {
		println!("File not exists: {}\n{ERR_ARGS_MESSAGE}", path_to_file_or_dir);
		return Ok(());
	}

	let mut file_names: Vec<String> = Vec::new();

	if arg_path.is_dir() {
		for entry in fs::read_dir(arg_path)? {
			let e = entry?;
			let path = e.path();
			if path.is_file() {
				file_names.push(path.to_string_lossy().to_string());
			}
		}
	} else if arg_path.is_file() {
		file_names.push(path_to_file_or_dir.clone());
	}

	let mut num_words_to_gen: i16 = 100;
	if args.len() > 2 {
		num_words_to_gen = atoi16(args[2].as_str());
		if num_words_to_gen < 2 {
			println!("{ERR_ARGS_MESSAGE}");
			return Ok(());
		}
	}

	let mut is_debug = false;

	for key in args {
		if key.to_lowercase().eq("--d") || key.to_lowercase().eq("--debug") {
			is_debug = true;
		}
	}

	let mut all_words: Vec<String> = Vec::new();
	
	let start_app_time = Local::now();
	println!("Start: {}", start_app_time.format("%Y-%m-%d %H:%M:%S%.3f"));

	let mut rng = rand::thread_rng();
	let mut start_time = Instant::now();

	for fname in &file_names {
		
		let lines_utf8_result = get_utf8_lines(&fname);
		
		let lines = match lines_utf8_result {
			Ok(l) => l,
			Err(_) => {
				let lines = get_1251_lines(&fname);
				match lines {
					Ok(l) => l,
					Err(_) => Vec::new()
				}
			}
		};

		add_lines_to_all_words(&lines, &mut all_words);
	}
	
	println!("file read: {}. Got {} words in {} files", duration_from(&start_time), all_words.len(), file_names.len());
	start_time = Instant::now();
	
    let mut pair_count: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for window in all_words.windows(2) {
        if let [first, second] = window {
            let entry = pair_count.entry(first.to_string()).or_insert_with(HashMap::new);
            *entry.entry(second.to_string()).or_insert(0) += 1;
        }
    }

	if is_debug {
		for (key, words) in &pair_count {
			println!("{}", key);
			for (word, count) in words {
				println!("  L__ {} ({})", word, count);
			}
		}
	}
	
	println!("got pair stat: {}. Total {} unique entries", duration_from(&start_time), pair_count.len());

	if all_words.len() < 1 {
		return Ok(());
	}

	start_time = Instant::now();
	
	let mut wrd: String;
	
	let n = rng.gen::<usize>() % all_words.len();
	wrd = all_words[n].clone();

	let mut sb_rnd_text = StringBuilder::new();
	sb_rnd_text.append(&wrd);
	sb_rnd_text.append(' ');

	if is_debug {
		println!("\n\nDEBUG:\n\n{wrd}");
	}

	let mut queue: VecDeque<String> = VecDeque::new();
	
	for _ in 0..num_words_to_gen {

		let mut num_repetitions = 0;
		for w in queue.iter() {
			if w.eq(&wrd) {
				num_repetitions += 1;
			}
		}

		if num_repetitions > 1 {
			let n = rng.gen::<usize>() % all_words.len();
			wrd = all_words[n].clone();
			if is_debug {
				print!("*  ");
			}
		}

		add_next_word(&pair_count, &mut wrd, &mut rng, &mut sb_rnd_text, is_debug);

		queue.push_back(wrd.clone());
        if queue.len() > 20 {
            queue.pop_front();
        }
	}

	for _ in 0..20 {
		let mut num_repetitions = 0;
		for w in queue.iter() {
			if w.eq(&wrd) {
				num_repetitions += 1;
			}
		}

		if num_repetitions > 2 {
			break;
		}

		add_next_word(&pair_count, &mut wrd, &mut rng, &mut sb_rnd_text, is_debug);

		if wrd.chars().last().unwrap_or_default() == '.' {
			break;
		}
	}

	println!("\n{}", sb_rnd_text.to_string());
	
	println!("\ngenerete and print random text: {}", duration_from(&start_time));
	
	Ok(())
}

fn add_next_word(pair_count: &HashMap<String, HashMap<String, usize>>, wrd: &mut String, 
	rng: &mut rand::prelude::ThreadRng, sb_rnd_text: &mut StringBuilder, is_debug: bool
) {
	match pair_count.get(&*wrd) {
		Some(&ref e) => {
			let total_wrd2_count: usize = e.values().sum();
			let n = rng.gen::<usize>() % total_wrd2_count;
			let mut sum_counts = 0;
			for (word, count) in e {
				sum_counts += count;
				if sum_counts >= n {
					*wrd = word.clone();
					sb_rnd_text.append(&*wrd);
					sb_rnd_text.append(' ');

					if is_debug {
						print!("{wrd}[{n}]  ");
					}
				
					break;
				}
			}
		},
		None => (),
	};
}

fn get_utf8_lines(fname: &String) -> Result<Vec<String>, io::Error> {
	let file = File::open(&fname)?;
	let reader = BufReader::new(file);
	let mut res = Vec::new();

	for line_result in reader.lines() {
		let line = line_result?;
        res.push(line);
	}
	Ok(res)
}

fn get_1251_lines(fname: &String) -> Result<Vec<String>, io::Error> {
	let file = File::open(&fname)?;

	let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

	let mut len_to_analyze = buffer.len();
	if len_to_analyze > 4096 {
		len_to_analyze = 4096;
	}

	let mut q_sym_1251 = 0;
	for i in 0..len_to_analyze {
		let c = buffer[i] as u8;
		if c >= 192 || c == 168 || c == 184 {
			q_sym_1251 += 1;
		}
	}

	let prc = (100 * q_sym_1251) / len_to_analyze;

	if prc < 60 {
		Ok(Vec::new())
	} else {
		let (decoded_content, _, _) = WINDOWS_1251.decode(&buffer);
		let res = decoded_content
			.lines()
			.map(|line| line.to_string())
			.collect();
		Ok(res)
	}
}

fn add_lines_to_all_words(lines: &Vec<String>, all_words: &mut Vec<String>) {
	for line in lines {
		for raw_word in line.split_whitespace() {
			let word: String = raw_word
				.to_lowercase()
				.chars()
				.filter(|c| *c != '(' && *c != ')' && *c != '«' && *c != '»' 
					&& *c != '\"' && *c != '[' && *c != ']' && *c != '<' && *c != '>')
				.collect();
			
			let index = word.find(|c: char| !c.is_alphabetic() && !is_russian_letter_lower(c) && c != '-')
				.unwrap_or(0);
			
			if has_russian_letter_lower(&word) && !word.is_empty() && word.len() < 30 {
				if index == 0 || index == word.len() - 1
					&& match word.chars().last() {
						Some(last_c) => last_c == '.' || last_c == ',' || last_c == '?' 
							|| last_c == '!' || last_c == ':' || last_c == ';' 
							|| last_c == ')',
						None => false
					}
				{
					all_words.push(word.to_string());
				}
			}
		}
	}
}

fn is_russian_letter_lower(c: char) -> bool {
    ('а'..='я').contains(&c)
}

fn has_russian_letter_lower(s: &String) -> bool {
	for c in s.chars() {
		if is_russian_letter_lower(c) {
			return true; 
		}
	}
	false
}

#[allow(dead_code)]
fn is_russian_letter(c: char) -> bool {
    ('а'..='я').contains(&c) || ('А'..='Я').contains(&c)
}

fn duration_from(from_time: &Instant) -> String  {
	let tm = Instant::now();
	let dur = tm.duration_since(*from_time);
	
	nanosec_to_str(dur.as_nanos())
}

fn nanosec_to_str(ns: u128) -> String {
	if ns > 1_000_000_000 {
		return format!("{:.2} sec", (ns as f64) / 1_000_000_000.0);
	}

	if ns > 1_000_000 {
		return format!("{:.2} ms", (ns as f64) / 1_000_000.0);
	}

	if ns > 1_000 {
		return format!("{:.2} mcs", (ns as f64) / 1_000.0);
	}

	format!("{} ns", ns)
}

fn atoi16(a: &str) -> i16 {
    match a.parse::<i16>() {
        Ok(n) => n,
        Err(_) => 0,
    }	
}
