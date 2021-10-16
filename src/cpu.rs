use std::process::exit;
use std::env;
use std::ffi::CString; use libc::{fopen, FILE};

#[derive(Debug)]
pub struct Cpu {
    ram: [u8; 512],
    r: [u64; 10],
    re: u64,
    pc: u64,
    sp: u64
}

impl Cpu {
    pub fn new(code: Vec<u8>) -> Cpu {
        let mut cpu = Cpu {
            ram: [0; 512],
            r: [0; 10],
            re: 0,
            pc: 0,
            sp: 512,
        };

        let mut i: usize = 0;

        for byte in code {
            cpu.ram[i] = byte;
            i += 1;
        }

        cpu
    }

    fn decode_str(&self, register: u64) -> String {
        let mut ptr: usize = register as usize;
        let mut result = String::from("");

        while self.ram[ptr] != 0x42 {
            result.push((self.ram[ptr] ^ 0x42) as char);

            ptr += 1;
        }

        result
    }

    fn push_str(&mut self, s: String) {
        let to_alloc = s.len() + 1;
        self.sp -= to_alloc as u64;

        for letter in s.chars() {
            self.ram[self.sp as usize] = letter as u8 ^ 0x42;
            self.sp += 1;
        }

        self.ram[self.sp as usize] = 0x42;
        self.sp -= to_alloc as u64;
        self.sp += 1;
    }

    fn find_id(&self, regid: u8) -> usize {
        match regid {
            0x03 => 1,
            0x05 => 2,
            0x08 => 3,
            0x0D => 4,
            0x15 => 5,
            0x22 => 6,
            0x37 => 7,
            0x59 => 8,
            0x90 => 9,
            0x89 => 10,
            _ => exit(66)
        }
    }

    pub fn run(&mut self) {
        loop {
            let inst = self.ram[self.pc as usize];
            
            match inst {
                0x04 => exit(0),
                
                0x02 => {
                    self.pc += 1;
                    let src = self.ram[self.pc as usize];
                    let var: String;
                    
                    if src == 1 {
                        var = self.decode_str(self.re);
                    }
                    else {
                        var = self.decode_str(self.r[self.find_id(src)]);
                    }

                    match env::var(var) {
                        Ok(data) => self.push_str(data),
                        Err(_e) => self.push_str(String::from("NONE"))
                    }

                    self.re = self.sp;
                    self.pc += 1;
                }
                
                0x07 => {
                    self.pc += 1;
                    let src = self.ram[self.pc as usize];

                    if src == 1 {
                        println!("{}", self.decode_str(self.re));
                    } 
                    else {
                        println!("{}", self.decode_str(self.r[self.find_id(src)]));
                    }
                    
                    self.pc += 1;
                }

                0x87 => {
                    /* TODO WRITE */
                }

                0x14 => {
                    self.pc += 1;
                    let r1 = self.ram[self.pc as usize];
                    let r2 = self.ram[self.pc as usize + 1];

                    let value: u64;

                    if r2 == 1 {
                        value = self.re;
                    }
                    else {
                        value = self.r[self.find_id(r2)];
                    }

                    if r1 == 1 {
                        self.re = value;
                    }
                    else {
                        self.r[self.find_id(r1)] = value;
                    }

                    self.pc += 2;
                }

                0x0C => {
                    self.pc += 1;
                    let s1 = self.ram[self.pc as usize];
                    let s2 = self.ram[self.pc as usize + 1];

                    let p1: u64;
                    let p2: u64;

                    if s1 == 1 {
                        p1 = self.re + 1;
                    }
                    else {
                        p1 = self.r[self.find_id(s1)];
                    }

                    if s2 == 1 {
                        p2 = self.re;
                    }
                    else {
                        p2 = self.r[self.find_id(s2)];
                    }

                    let s = self.decode_str(p1) + &self.decode_str(p2);
                    self.push_str(s);
                   
                    self.re = self.sp;
                    self.pc += 2;
                }

                0xe8 => {
                    self.re = self.pc + 1;

                    while self.ram[self.pc as usize] != 0x42 {
                        self.pc += 1
                    }

                    self.pc += 1;
                }

                0x21 => {
                    self.pc += 1;
                    let _dst = self.ram[self.pc as usize];
                    let src = self.ram[self.pc as usize + 1];

                    let s: String;

                    if src == 1 {
                        s = self.decode_str(self.re);
                    }
                    else {
                        s = self.decode_str(self.r[self.find_id(src)]);
                    }

                    let filename = CString::new(s).unwrap();
                    let read = CString::new("a").unwrap();

                    unsafe {
                        let fp: *mut FILE = fopen(filename.as_ptr(), read.as_ptr());
                        self.re = fp as u64;
                    }

                    self.pc += 2;
                }

                _ => {
                    println!("{:x}", inst);
                    exit(255);
                }
            }
        }
    }
}
