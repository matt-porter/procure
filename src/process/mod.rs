//! Process Metrics
use std::fs;
use std::fs::File;
use std::io::Read;
use std::iter::Iterator;

use csv::Reader;

use rustc_serialize::{Decoder, Decodable};

#[derive(Debug)]
pub struct Process {
    pub pid: i32,
    pub command: String,
    pub start_time: u64,
    pub rss: i32,
    pub vsz: u64,
    pub cpu_time: u64,
    pub cpu_percent: f32,
}

#[derive(RustcDecodable)]
struct ProcessStat {
    pid: i32,
    comm: String,
    state: char,
    ppid: i32,
    pgrp: i32,
    session: i32,
    tty_nr: i32,
    tpgid: i32,
    flags: u32,
    minflt: u64,
    cminflt: u64,
    majflt: u64,
    cmajflt: u64,
    utime: u64,
    stime: u64,
    cutime: i64,
    cstime: i64,
    priority: i64,
    nice: i64,
    num_threads: i64,
    itrealvalue: i64,
    starttime: u64,
    vsize: u64,
    rss: i32,
    rsslim: u64,
    startcode: u64,
    endcode: u64,
    startstack: u64,
    kstkesp: u64,
    kstkeip: u64,
    signal: u64,
    blocked: u64,
    sigignore: u64,
    sigcatch: u64,
    wchan: u64,
    nswap: u64,
    cnswap: u64,
    exit_signal: i32,
    processor: i32,
    rt_priority: u32,
    policy: u32,
    delayacct_blkio_ticks: u64, // llu?
    guest_time: u64,
    cguesttime: i64,
    start_data: u64,
    end_data: u64,
    start_brk: u64,
    arg_start: u64,
    arg_end: u64,
    env_start: u64,
    env_end: u64,
    exit_code: u64,
}

pub struct Processes {
    iter: Box<Iterator<Item = Process>>,
}

impl Iterator for Processes {
    type Item = Process;

    fn next(&mut self) -> Option<Process> {
        self.iter.next()
    }
}

pub struct Pids {
    iter: Box<Iterator<Item = i32>>,
}

impl Iterator for Pids {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.iter.next()
    }
}

fn pids_from_path(proc_path: &str) -> Pids {
    let iter = fs::read_dir(proc_path).unwrap()
        // Process directories might have gone away since
        // the directory was read. It's fine to ignore those.
        .filter_map(|entry| entry.ok())
        // Map entry to a string, removing it if it fails to
        // parse as unicode.
        .filter_map(|entry| entry.file_name().into_string().ok())
        // Remove any entries that can't be converted to an integer.
        .filter_map(|entry| entry.parse::<i32>().ok());
    Pids { iter: Box::new(iter) }
}

fn processes_from_path(proc_path: &str) -> Processes {
    let pids = pids_from_path(proc_path);
    let processes: Vec<Result<Process, &'static str>> =
        pids.map(|pid| process_from_path(proc_path, pid))
            .collect();
    Processes { iter: Box::new(processes.into_iter().filter_map(|p| p.ok()).into_iter()) }
}

fn read_stat_file(path: &str) -> ProcessStat {
    Reader::from_file(path)
        .expect("Failed to open file")
        .has_headers(false)
        .delimiter(' ' as u8)
        .decode()
        .filter_map(|stat| stat.ok())
        .next()
        .unwrap()
}

fn process_from_path(proc_path: &str, pid: i32) -> Result<Process, &'static str> {
    // Gather the process data present in "`path`/`pid`".
    let mut command = String::new();
    let mut f = File::open(&format!("{}/{}/cmdline", proc_path, pid)).expect("Failed to open path");
    f.read_to_string(&mut command).expect("Failed to read file");
    let stat = read_stat_file(&format!("{}/{}/stat", proc_path, pid));
    Ok(Process {
        pid: pid,
        command: command,
        start_time: stat.starttime,
        rss: stat.rss,
        vsz: stat.vsize,
        cpu_time: stat.utime + stat.stime,
        cpu_percent: 0f32,
    })
}

// Public interface

pub fn pids() -> Pids {
    pids_from_path("/proc")
}

pub fn processes() -> Processes {
    processes_from_path("/proc")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pids_from_path() {
        let mut pids: Vec<i32> = super::pids_from_path("testdata/proc").collect();
        pids.sort();
        assert_eq!(pids, vec![1, 16018, 24064, 24126]);
    }

    #[test]
    fn test_processes_from_path() {
        let mut processes: Vec<Process> = super::processes_from_path("testdata/proc").collect();
        processes.sort_by_key(|p| p.pid);
        for (i, pid) in (0..).zip(vec![1, 16018, 24064, 24126].into_iter()) {
            println!("{:?}", processes[i]);
            assert_eq!(processes[i].pid, pid);
        }
    }

    #[test]
    fn test_process_from_path() {
        let process = super::process_from_path("testdata/proc", 1);
        // assert!(process.name == "init");
        // assert!(process.VmRSS == 2164);
        assert!(process.unwrap().command == "/sbin/init");
    }
}
