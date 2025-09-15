use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::process::Command;
use std::process::Child;
use std::fs;
use std::fs::OpenOptions;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::thread;
fn main() {

    let db_name = "process_list.db";

    let mut process_list : Vec<Process> = Vec::new();

    if fs::exists(db_name).unwrap() {

        let process_db = fs::read_to_string(db_name).unwrap();

        for process in process_db.split("\r\n"){ 
            let args : Vec<&str> = process.trim().split(" ").collect();

            if args.len() == 2 {
                let mut process = Process::new(
                    args[0].to_string(),
                    args[1].to_string(),
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                );

                process.spawn();

                process_list.push(process);
            }
        }
    }else{
        fs::write(db_name, b"").unwrap();
    }

    loop {

        print!("> ");

        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();

        std::io::stdin().read_line(&mut buffer).unwrap();

        let args : Vec<&str> = buffer.trim().split(" ").collect();

        if args.len() > 0 {

            match args[0]{
                "start" => {
                    if args.len() == 3 {
                        let mut process = Process::new(
                            args[1].to_string(),
                            args[2].to_string(), 
                            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                        );

                        process.spawn();

                        process_list.push(process);

                        let mut db_data = String::new();

                        for process in process_list.iter() {
                            
                            db_data.push_str(&format!(
                                "{} {}\r\n",
                                process.file_name,
                                process.name
                            )); 
                        }

                        std::fs::write(db_name, db_data.as_bytes()).unwrap();

                    }else{
                        println!("Not enough args");
                    }
                },
                "status" => {
                    if process_list.len() == 7 {
                        println!("Process list is empty");
                    }else{

                        let reset = "\x1b[0m";
                        let bold = "\x1b[1m\x1b[32m";
                        let top_left = "┌";
                        let top_right = "┐";
                        let bottom_left = "└";
                        let bottom_right = "┘";
                        let horizontal = "─";
                        let vertical = "│";
                        let junction_top = "┬";
                        let junction_bottom = "┴";
                        let junction_left = "├";
                        let junction_right = "┤";
                        let cross = "┼";

                        // Table header
                        println!("{top_left}{hz1}{junction_top}{hz1}{junction_top}{hz2}{junction_top}{hz2}{junction_top}{hz1}{top_right}", 
                            hz1 = horizontal.repeat(10), 
                            hz2 = horizontal.repeat(50)
                        );

                        println!("{vt} {bold}{:<8}{reset} {vt} {bold}{:<8}{reset} {vt} {bold}{:<48}{reset} {vt} {bold}{:<48}{reset} {vt} {bold}{:<8}{reset} {vt}","ID", "PID", "Name", "File", "Active", 
                            vt = vertical
                        );

                        println!("{junction_left}{hz1}{cross}{hz1}{cross}{hz2}{cross}{hz2}{cross}{hz1}{junction_right}", 
                            hz1 = horizontal.repeat(10), 
                            hz2 = horizontal.repeat(50)
                        );

                        for (index,process) in process_list.iter().enumerate() {
                            
                            if let Some(child) = &process.child {

                                let elapsed_time = format!("{}s",SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - process.time);
                                
                                println!("{vt} {:<8} {vt} {:<8} {vt} {:<48} {vt} {:<48} {vt} {:<8} {vt}", 
                                    index,
                                    child.id(), 
                                    process.name, 
                                    process.file_name, 
                                    elapsed_time, 
                                    vt = vertical
                                );

                                if index == process_list.len() -1 {
                                    println!("{bottom_left}{hz1}{junction_bottom}{hz1}{junction_bottom}{hz2}{junction_bottom}{hz2}{junction_bottom}{hz1}{bottom_right}", 
                                        hz1 = horizontal.repeat(10), 
                                        hz2 = horizontal.repeat(50)
                                    ); 
                                }else{ 
                                    println!("{junction_left}{hz1}{cross}{hz1}{cross}{hz2}{cross}{hz2}{cross}{hz1}{junction_right}", 
                                        hz1 = horizontal.repeat(10), 
                                        hz2 = horizontal.repeat(50)
                                    );
                                }
                            }
                        }

                    }
                }
                "remove" => {
                    if args.len() == 2 {

                        let index = args[1].parse::<usize>().unwrap();

                        if let Some(process) = process_list.get_mut(index){
                            if let Some(child) = process.child.as_mut() {
                                child.kill().unwrap();
                                child.wait().unwrap();

                                process_list.remove(index);
                            }  
                        }
                    }else{
                        println!("Missing process index")
                    }
                },
                "logs" => {
                    if args.len() == 2 {

                        let index = args[1].parse::<usize>().unwrap();

                        if let Some(process) = process_list.get(index){
                            let name = process.name.clone();

                            let data = fs::read_to_string(format!("{}.log", name))
                                .unwrap();

                            println!("{}", data);
                        }
                    }else{
                        println!("Missing process index")
                    }
                },
                "quit" => {
                    std::process::exit(0);
                }, 
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }
}

#[derive(Debug)]
struct Process {
    file_name : String,
    name : String,
    time : u64,
    child : Option<Child>
}

impl Process {

    fn new(file_name : String, name : String, time : u64) -> Self {
        Self {
            file_name,
            name,
            time,
            child : None
        }
    }

    fn spawn(&mut self){

        let mut child = Command::new("node")
            .arg(self.file_name.clone())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take();

        self.child = Some(child);

        if let Some(stdout) = stdout {

            let name = self.name.clone();

            thread::spawn(move || {

                let buffer = BufReader::new(stdout);

                for result in buffer.lines(){
                    let line = format!("{}\n", result.unwrap());

                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(format!("{}.log", name))
                        .unwrap();

                    file.write(line.as_bytes()).unwrap();
                }

            });

        }
    }
}