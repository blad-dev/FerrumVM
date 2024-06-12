use num::traits::ToBytes;
use num::{cast::AsPrimitive, traits::bounds::UpperBounded};
use std::collections::{hash_map, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Output;
use std::vec;
struct BufferArray {
    buffer: [u8; 100_000],
}
impl BufferArray {
    fn new() -> BufferArray {
        BufferArray {
            buffer: [0; 100_000],
        }
    }
}
impl Buffer for BufferArray {
    fn load<T>(&self, id: usize) -> T {
        unsafe { (self.buffer.as_ptr().add(id) as *const T).read_unaligned() }
    }
    fn store<T>(&mut self, id: usize, value: T) -> () {
        unsafe {
            (self.buffer.as_mut_ptr().add(id) as *mut T).write_unaligned(value);
        }
    }
}
trait Buffer {
    fn load<T>(&self, id: usize) -> T;
    fn store<T>(&mut self, id: usize, value: T) -> ();
}
trait StackMachine {
    fn push<T>(&mut self, value: T) -> ();
    fn pop<T>(&mut self) -> T;
    fn peek<T>(&self) -> T;

    fn add<T: std::ops::AddAssign>(&mut self) -> ();
    fn subtract<T: std::ops::SubAssign>(&mut self) -> ();
    fn multiply<T: std::ops::MulAssign>(&mut self) -> ();
    fn divide<T: std::ops::DivAssign>(&mut self) -> ();

    fn logic_and(&mut self) -> ();
    fn logic_or(&mut self) -> ();
    fn logic_not(&mut self) -> ();

    fn compare_equal<T: std::cmp::PartialOrd>(&mut self) -> ();
    fn compare_not_equal<T: std::cmp::PartialOrd>(&mut self) -> ();

    fn compare_greater<T: std::cmp::PartialOrd>(&mut self) -> ();
    fn compare_greater_equal<T: std::cmp::PartialOrd>(&mut self) -> ();

    fn compare_lesser<T: std::cmp::PartialOrd>(&mut self) -> ();
    fn compare_lesser_equal<T: std::cmp::PartialOrd>(&mut self) -> ();

    fn cast_from_to<From: AsPrimitive<To>, To: 'static + Copy>(&mut self) -> ();

    fn store<StoreType, T: Buffer>(&mut self, buffer: &mut T, id: usize) -> ();
    fn peek_store<StoreType, T: Buffer>(&self, buffer: &mut T, id: usize) -> ();
    fn load<StoreType, T: Buffer>(&mut self, buffer: &T, id: usize) -> ();
}
struct StackArray {
    stack: [u8; 10_000],
    end: *mut u8,
}
macro_rules! cast_type_to_type_same_size {
    ($function_name:ident, $type_from: ty, $type_to: ty) => {
        fn $function_name(&mut self) -> () {
            unsafe {
                let pointer: *mut u8 = self.end.sub(std::mem::size_of::<$type_from>());
                let value: $type_from = (pointer as *const $type_from).read_unaligned();
                (pointer as *mut $type_to).write_unaligned(value as $type_to);
            }
        }
    };
}
impl StackArray {
    fn new() -> StackArray {
        let mut stack = StackArray {
            stack: [0u8; 10_000],
            end: 0 as *mut u8,
        };
        stack.end = stack.stack.as_mut_ptr();
        stack
    }
    fn init(&mut self) -> () {
        self.end = self.stack.as_mut_ptr();
    }
}
impl StackMachine for StackArray {
    fn push<T>(&mut self, value: T) -> () {
        unsafe {
            (self.end as *mut T).write_unaligned(value);
            self.end = self.end.add(std::mem::size_of::<T>());
        }
    }
    fn pop<T>(&mut self) -> T {
        unsafe {
            self.end = self.end.sub(std::mem::size_of::<T>());
            (self.end as *const T).read_unaligned()
        }
    }
    fn peek<T>(&self) -> T {
        unsafe { (self.end.sub(std::mem::size_of::<T>()) as *const T).read_unaligned() }
    }

    fn add<T: std::ops::AddAssign>(&mut self) -> () {
        let value = self.pop::<T>();
        unsafe {
            *(self.end.sub(std::mem::size_of::<T>()) as *mut T) += value;
        }
    }
    fn subtract<T: std::ops::SubAssign>(&mut self) -> () {
        let value = self.pop::<T>();
        unsafe {
            *(self.end.sub(std::mem::size_of::<T>()) as *mut T) -= value;
        }
    }
    fn multiply<T: std::ops::MulAssign>(&mut self) -> () {
        let value = self.pop::<T>();
        unsafe {
            *(self.end.sub(std::mem::size_of::<T>()) as *mut T) *= value;
        }
    }
    fn divide<T: std::ops::DivAssign>(&mut self) -> () {
        let value = self.pop::<T>();
        unsafe {
            *(self.end.sub(std::mem::size_of::<T>()) as *mut T) /= value;
        }
    }

    fn logic_and(&mut self) -> () {
        let value = self.pop::<bool>();
        unsafe {
            *(self.end.sub(std::mem::size_of::<bool>()) as *mut bool) &= value;
        }
    }
    fn logic_or(&mut self) -> () {
        let value = self.pop::<bool>();
        unsafe {
            *(self.end.sub(std::mem::size_of::<bool>()) as *mut bool) |= value;
        }
    }
    fn logic_not(&mut self) -> () {
        unsafe {
            let ptr = self.end.sub(std::mem::size_of::<bool>()) as *mut bool;
            *ptr = !(*ptr);
        }
    }

    fn compare_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        let value1 = self.pop::<T>();
        let value2 = self.pop::<T>();
        self.push::<bool>(value2 == value1);
    }

    fn compare_not_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        let value1 = self.pop::<T>();
        let value2 = self.pop::<T>();
        self.push::<bool>(value2 != value1);
    }

    fn compare_greater<T: std::cmp::PartialOrd>(&mut self) -> () {
        let value1 = self.pop::<T>();
        let value2 = self.pop::<T>();
        self.push::<bool>(value2 > value1);
    }
    fn compare_greater_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        let value1 = self.pop::<T>();
        let value2 = self.pop::<T>();
        self.push::<bool>(value2 >= value1);
    }

    fn compare_lesser<T: std::cmp::PartialOrd>(&mut self) -> () {
        let value1 = self.pop::<T>();
        let value2 = self.pop::<T>();
        self.push::<bool>(value2 < value1);
    }
    fn compare_lesser_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        let value1 = self.pop::<T>();
        let value2 = self.pop::<T>();
        self.push::<bool>(value2 <= value1);
    }

    fn cast_from_to<From: AsPrimitive<To>, To: 'static + Copy>(&mut self) -> () {
        unsafe {
            let pointer: *mut u8 = self.end.sub(std::mem::size_of::<From>());
            let value: From = (pointer as *const From).read_unaligned();
            (pointer as *mut To).write_unaligned(value.as_());
            let OFFSET: isize =
                std::mem::size_of::<To>() as isize - std::mem::size_of::<From>() as isize;
            self.end = self.end.offset(OFFSET);
        }
    }
    fn store<StoreType, T: Buffer>(&mut self, buffer: &mut T, id: usize) -> () {
        buffer.store::<StoreType>(id, self.pop::<StoreType>());
    }
    fn peek_store<StoreType, T: Buffer>(&self, buffer: &mut T, id: usize) -> () {
        buffer.store::<StoreType>(id, self.peek::<StoreType>());
    }
    fn load<StoreType, T: Buffer>(&mut self, buffer: &T, id: usize) -> () {
        self.push::<StoreType>(buffer.load::<StoreType>(id));
    }
}
fn create_mapping() -> HashMap<String, u8> {
    HashMap::from([
        ("push".to_owned(), Token::Push as u8),
        ("pop".to_owned(), Token::Pop as u8),
        ("peek".to_owned(), Token::Peek as u8),
        ("clone_push".to_owned(), Token::ClonePush as u8),
        ("add".to_owned(), Token::Add as u8),
        ("subtract".to_owned(), Token::Subtract as u8),
        ("multiply".to_owned(), Token::Multiply as u8),
        ("divide".to_owned(), Token::Divide as u8),
        ("store".to_owned(), Token::Store as u8),
        ("peek_store".to_owned(), Token::PeekStore as u8),
        ("load".to_owned(), Token::Load as u8),
        ("goto".to_owned(), Token::Goto as u8),
        ("pop_goto_if_true".to_owned(), Token::PopGotoIfTrue as u8),
        ("peek_goto_if_true".to_owned(), Token::PeekGotoIfTrue as u8),
        ("logic_and".to_owned(), Token::LogicAnd as u8),
        ("logic_or".to_owned(), Token::LogicOr as u8),
        ("logic_not".to_owned(), Token::LogicNot as u8),
        ("compare_equal".to_owned(), Token::CompareEqual as u8),
        ("compare_not_equal".to_owned(), Token::CompareNotEqual as u8),
        ("compare_greater".to_owned(), Token::CompareGreater as u8),
        (
            "compare_greater_equal".to_owned(),
            Token::CompareGreaterEqual as u8,
        ),
        ("compare_lesser".to_owned(), Token::CompareLesser as u8),
        (
            "compare_lesser_equal".to_owned(),
            Token::CompareLesserEqual as u8,
        ),
        ("type_cast".to_owned(), Token::TypeCast as u8),
        ("bool".to_owned(), Token::Bool as u8),
        ("i8".to_owned(), Token::I8 as u8),
        ("i16".to_owned(), Token::I16 as u8),
        ("i32".to_owned(), Token::I32 as u8),
        ("i64".to_owned(), Token::I64 as u8),
        ("u8".to_owned(), Token::U8 as u8),
        ("u16".to_owned(), Token::U16 as u8),
        ("u32".to_owned(), Token::U32 as u8),
        ("u64".to_owned(), Token::U64 as u8),
        ("f32".to_owned(), Token::F32 as u8),
        ("f64".to_owned(), Token::F64 as u8),
    ])
}
fn try_parse_value(number_token: u8, string_val: &str) -> (i32, [u8; 8]) {
    let mut arr = [0u8; 8];
    const Bool: u8 = Token::Bool as u8;
    const I8: u8 = Token::I8 as u8;
    const I16: u8 = Token::I16 as u8;
    const I32: u8 = Token::I32 as u8;
    const I64: u8 = Token::I64 as u8;
    const U8: u8 = Token::U8 as u8;
    const U16: u8 = Token::U16 as u8;
    const U32: u8 = Token::U32 as u8;
    const U64: u8 = Token::U64 as u8;
    const F32: u8 = Token::F32 as u8;
    const F64: u8 = Token::F64 as u8;

    match number_token {
        Bool => {
            if string_val == "true" {
                arr[0] = 1;
                return (1, arr);
            }
            else if string_val == "false"{
                arr[0] = 0;
                return (1, arr);
            }
            else{
                panic!("Unexpected string! {}", string_val);
            }
        },
        I8 => {
            let val = string_val.parse::<i8>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut i8).write_unaligned(val);
            }
            return (std::mem::size_of::<i8>() as i32, arr);
        },
        I16 => {
            let val = string_val.parse::<i16>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut i16).write_unaligned(val);
            }
            return (std::mem::size_of::<i16>() as i32, arr);
        },
        I32 => {
            let val = string_val.parse::<i32>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut i32).write_unaligned(val);
            }
            return (std::mem::size_of::<i32>() as i32, arr);
        },
        I64 => {
            let val = string_val.parse::<i64>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut i64).write_unaligned(val);
            }
            return (std::mem::size_of::<i64>() as i32, arr);
        },
        U8 => {
            let val = string_val.parse::<u8>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut u8).write_unaligned(val);
            }
            return (std::mem::size_of::<u8>() as i32, arr);
        },
        U16 => {
            let val = string_val.parse::<u16>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut u16).write_unaligned(val);
            }
            return (std::mem::size_of::<u16>() as i32, arr);
        },
        U32 => {
            let val = string_val.parse::<u32>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut u32).write_unaligned(val);
            }
            return (std::mem::size_of::<u32>() as i32, arr);
        },
        U64 => {
            let val = string_val.parse::<u64>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut u64).write_unaligned(val);
            }
            return (std::mem::size_of::<u64>() as i32, arr);
        },
        F32 => {
            let val = string_val.parse::<f32>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut f32).write_unaligned(val);
            }
            return (std::mem::size_of::<f32>() as i32, arr);
        },
        F64 => {
            let val = string_val.parse::<f64>().expect(&format!("The parse failed! {}", string_val));
            unsafe {
                (arr.as_mut_ptr() as *mut f64).write_unaligned(val);
            }
            return (std::mem::size_of::<f64>() as i32, arr);
        },
        _ => {panic!("The wrong token passed! {}", number_token)},
    }
}
fn parse_to_vector(path: &str) -> Vec<u8> {
    let mut prev_token: u8 = 0;
    let mut output = Vec::<u8>::new();
    let hash_map = create_mapping();
    let reader = BufReader::new(File::open(path).expect("Cannot open file.txt"));
    for line in reader.lines() {
        for word in line.unwrap().split_whitespace() {
            let option = hash_map.get(word);
            if (option.is_none()) {
                if (prev_token >= 24 && prev_token <= 34) {
                    let value = try_parse_value(prev_token, word);
                    for i in 0..value.0{
                        output.push(value.1[i as usize]);
                    }
                    continue;
                }
                else if(prev_token >= 11 && prev_token <= 13){
                    let mut arr = [0u8; 8];
                    unsafe{
                        (arr.as_mut_ptr() as *mut usize).write_unaligned(word.parse::<usize>().expect(&format!("Unexpected token {}", word)));
                    }
                    for i in 0..8{
                        output.push(arr[i as usize]);
                    }
                    continue;
                }
                else{
                    panic!("Unexpected token: {}", word);

                }
            }
            let result = option.unwrap();

            output.push(*result);
            prev_token = *result;
        }
    }
    return output;
}
fn main() {
    let mut stack = StackUpperVector::new();

    stack.token_byte_sequence = parse_to_vector("./data/file2.txt");
    /*stack.token_byte_sequence = vec![
        Token::Push as u8,
        Token::Bool as u8,
        0,
        Token::PopGotoIfTrue as u8,
        48,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        Token::Push as u8,
        Token::F32 as u8,
        1,
        0,
        0,
        0,
        Token::Peek as u8,
        Token::I32 as u8,
        Token::Push as u8,
        Token::I32 as u8,
        0,
        1,
        0,
        0,
        Token::Peek as u8,
        Token::I32 as u8,
        Token::Add as u8,
        Token::I32 as u8,
        Token::TypeCast as u8,
        Token::I32 as u8,
        Token::F64 as u8,
        Token::Peek as u8,
        Token::F64 as u8,
        Token::Push as u8,
        Token::I32 as u8,
        26,
        0,
        0,
        0,
        Token::TypeCast as u8,
        Token::I32 as u8,
        Token::F64 as u8,
        Token::Divide as u8,
        Token::F64 as u8,
        Token::Pop as u8,
        Token::F64 as u8,
        Token::Push as u8,
        Token::I32 as u8,
        101,
        0,
        0,
        0,
        Token::Pop as u8,
        Token::I32 as u8,
    ];*/
    /*stack.token_byte_sequence = vec![
        Token::Push as u8,
        Token::I32 as u8,
        0,
        0,
        0,
        0,
        Token::ClonePush as u8,
        Token::I32 as u8,
        Token::Push as u8,
        Token::I32 as u8,
        10,
        0,
        0,
        0,
        Token::CompareGreaterEqual as u8,
        Token::I32 as u8,
        Token::PopGotoIfTrue as u8,
        44,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        Token::Push as u8,
        Token::I32 as u8,
        1,
        0,
        0,
        0,
        Token::Add as u8,
        Token::I32 as u8,
        Token::Peek as u8,
        Token::I32 as u8,
        Token::Goto as u8,
        6,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];*/

    /*let reader = BufReader::new(File::open("./data/file.txt").expect("Cannot open file.txt"));
    for line in reader.lines() {
        for word in line.unwrap().split_whitespace() {
            println!("word '{}'", word);
        }
        println!("");

    }*/
    stack.init();
    stack.execute_all();
    //tests();
}
struct StackUpperVector {
    lower_stack: StackArray,
    buffer: BufferArray,
    token_byte_sequence: Vec<u8>,
    cursor: *mut u8,
}
macro_rules! match_all_types {
    ($operation: ident, $self: expr) => {
        let _type = $self.get::<u8>();
        match _type {
            Bool => {
                $self.$operation::<bool>();
            }
            I8 => {
                $self.$operation::<i8>();
            }
            I16 => {
                $self.$operation::<i16>();
            }
            I32 => {
                $self.$operation::<i32>();
            }
            I64 => {
                $self.$operation::<i64>();
            }
            U8 => {
                $self.$operation::<u8>();
            }
            U16 => {
                $self.$operation::<u16>();
            }
            U32 => {
                $self.$operation::<u32>();
            }
            U64 => {
                $self.$operation::<u64>();
            }
            F32 => {
                $self.$operation::<f32>();
            }
            F64 => {
                $self.$operation::<f64>();
            }
            _ => {
                panic!("Unknown type")
            }
        }
    };
}
macro_rules! match_all_numeric_types {
    ($operation: ident, $self: expr) => {
        let _type = $self.get::<u8>();
        match _type {
            I8 => {
                $self.$operation::<i8>();
            }
            I16 => {
                $self.$operation::<i16>();
            }
            I32 => {
                $self.$operation::<i32>();
            }
            I64 => {
                $self.$operation::<i64>();
            }
            U8 => {
                $self.$operation::<u8>();
            }
            U16 => {
                $self.$operation::<u16>();
            }
            U32 => {
                $self.$operation::<u32>();
            }
            U64 => {
                $self.$operation::<u64>();
            }
            F32 => {
                $self.$operation::<f32>();
            }
            F64 => {
                $self.$operation::<f64>();
            }
            _ => {
                panic!("Unknown type")
            }
        }
    };
}
macro_rules! cast_from_to_types {
    ($case: ident, $type: ty, $match_type: expr) => {
        $case => {
            match $match_type {
                I8 => self.lower_stack.cast_from_to::<$type, i8>(),
                I16 => self.lower_stack.cast_from_to::<$type, i16>(),
                I32 => self.lower_stack.cast_from_to::<$type, i32>(),
                I64 => self.lower_stack.cast_from_to::<$type, i64>(),

                U8 =>  self.lower_stack.cast_from_to::<$type, u8>(),
                U16 => self.lower_stack.cast_from_to::<$type, u16>(),
                U32 => self.lower_stack.cast_from_to::<$type, u32>(),
                U64 => self.lower_stack.cast_from_to::<$type, u64>(),

                F32 => self.lower_stack.cast_from_to::<$type, f32>(),
                F64 => self.lower_stack.cast_from_to::<$type, f64>(),

                _ => panic!("Invalid type cast!"),
            }
        }
    };
}

impl StackUpperVector {
    fn new() -> StackUpperVector {
        StackUpperVector {
            lower_stack: StackArray::new(),
            buffer: BufferArray::new(),
            token_byte_sequence: Vec::new(),
            cursor: 0 as *mut u8,
        }
    }
    fn init(&mut self) -> () {
        self.lower_stack.init();
        self.cursor = self.token_byte_sequence.as_mut_ptr();
    }
    fn goto(&mut self, cursor_bytes_id: usize) -> () {
        unsafe {
            self.cursor = self.token_byte_sequence.as_mut_ptr().add(cursor_bytes_id);
        }
    }
    fn get<T>(&mut self) -> T {
        unsafe {
            let value = (self.cursor as *const T).read_unaligned();
            self.cursor = self.cursor.add(std::mem::size_of::<T>());
            value
        }
    }
    fn push<T>(&mut self) -> () {
        let value = self.get::<T>();
        self.lower_stack.push::<T>(value);
    }
    fn pop<T: std::fmt::Display>(&mut self) -> () {
        println!("{:.3}", self.lower_stack.pop::<T>());
    }
    fn peek<T: std::fmt::Display>(&mut self) -> () {
        println!("{:.3}", self.lower_stack.peek::<T>());
    }
    fn add<T: std::ops::AddAssign>(&mut self) -> () {
        self.lower_stack.add::<T>();
    }
    fn subtract<T: std::ops::SubAssign>(&mut self) -> () {
        self.lower_stack.subtract::<T>();
    }
    fn multiply<T: std::ops::MulAssign>(&mut self) -> () {
        self.lower_stack.multiply::<T>();
    }
    fn divide<T: std::ops::DivAssign>(&mut self) -> () {
        self.lower_stack.divide::<T>();
    }

    fn compare_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        self.lower_stack.compare_equal::<T>();
    }
    fn compare_not_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        self.lower_stack.compare_not_equal::<T>();
    }
    fn compare_greater<T: std::cmp::PartialOrd>(&mut self) -> () {
        self.lower_stack.compare_greater::<T>();
    }
    fn compare_greater_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        self.lower_stack.compare_greater_equal::<T>();
    }
    fn compare_lesser<T: std::cmp::PartialOrd>(&mut self) -> () {
        self.lower_stack.compare_lesser::<T>();
    }
    fn compare_lesser_equal<T: std::cmp::PartialOrd>(&mut self) -> () {
        self.lower_stack.compare_lesser_equal::<T>();
    }
    fn store<T>(&mut self) -> () {
        let id = self.get::<usize>();
        self.lower_stack
            .store::<T, BufferArray>(&mut self.buffer, id);
    }
    fn peek_store<T>(&mut self) -> () {
        let id = self.get::<usize>();
        self.lower_stack
            .peek_store::<T, BufferArray>(&mut self.buffer, id);
    }
    fn load<T>(&mut self) -> () {
        let id = self.get::<usize>();
        self.lower_stack
            .load::<T, BufferArray>(&mut self.buffer, id);
    }
    fn clone_push<T>(&mut self) -> () {
        let value = self.lower_stack.peek::<T>();
        self.lower_stack.push::<T>(value);
    }
    fn do_Token(&mut self) -> () {
        let Token = self.get::<u8>();
        const Push: u8 = Token::Push as u8;
        const Pop: u8 = Token::Pop as u8;
        const Peek: u8 = Token::Peek as u8;
        const ClonePush: u8 = Token::ClonePush as u8;
        const Add: u8 = Token::Add as u8;
        const Subtract: u8 = Token::Subtract as u8;
        const Multiply: u8 = Token::Multiply as u8;
        const Divide: u8 = Token::Divide as u8;
        const Store: u8 = Token::Store as u8;
        const PeekStore: u8 = Token::PeekStore as u8;
        const Load: u8 = Token::Load as u8;
        const Goto: u8 = Token::Goto as u8;
        const PopGotoIfTrue: u8 = Token::PopGotoIfTrue as u8;
        const PeekGotoIfTrue: u8 = Token::PeekGotoIfTrue as u8;
        const LogicAnd: u8 = Token::LogicAnd as u8;
        const LogicOr: u8 = Token::LogicOr as u8;
        const LogicNot: u8 = Token::LogicNot as u8;
        const TypeCast: u8 = Token::TypeCast as u8;
        const CompareEqual: u8 = Token::CompareEqual as u8;
        const CompareNotEqual: u8 = Token::CompareNotEqual as u8;
        const CompareGreater: u8 = Token::CompareGreater as u8;
        const CompareGreaterEqual: u8 = Token::CompareGreaterEqual as u8;
        const CompareLesser: u8 = Token::CompareLesser as u8;
        const CompareLesserEqual: u8 = Token::CompareLesserEqual as u8;

        const Bool: u8 = Token::Bool as u8;
        const I8: u8 = Token::I8 as u8;
        const I16: u8 = Token::I16 as u8;
        const I32: u8 = Token::I32 as u8;
        const I64: u8 = Token::I64 as u8;
        const U8: u8 = Token::U8 as u8;
        const U16: u8 = Token::U16 as u8;
        const U32: u8 = Token::U32 as u8;
        const U64: u8 = Token::U64 as u8;
        const F32: u8 = Token::F32 as u8;
        const F64: u8 = Token::F64 as u8;

        match Token {
            Push => {
                match_all_types!(push, self);
            }
            Pop => {
                match_all_types!(pop, self);
            }
            Peek => {
                match_all_types!(peek, self);
            }
            ClonePush => {
                match_all_types!(clone_push, self);
            }
            Add => {
                match_all_numeric_types!(add, self);
            }
            Subtract => {
                match_all_numeric_types!(subtract, self);
            }
            Multiply => {
                match_all_numeric_types!(multiply, self);
            }
            Divide => {
                match_all_numeric_types!(divide, self);
            }

            Store => {
                match_all_types!(store, self);
            }
            PeekStore => {
                match_all_types!(peek_store, self);
            }
            Load => {
                match_all_types!(load, self);
            }

            Goto => {
                let cursor_bytes_id = self.get::<usize>();
                self.goto(cursor_bytes_id);
            }
            PopGotoIfTrue => {
                let cursor_bytes_id = self.get::<usize>();
                if self.lower_stack.pop::<bool>() {
                    self.goto(cursor_bytes_id);
                }
            }
            PeekGotoIfTrue => {
                let cursor_bytes_id = self.get::<usize>();
                if self.lower_stack.peek::<bool>() {
                    self.goto(cursor_bytes_id);
                }
            }

            LogicAnd => {
                self.lower_stack.logic_and();
            }
            LogicOr => {
                self.lower_stack.logic_or();
            }
            LogicNot => {
                self.lower_stack.logic_not();
            }

            CompareEqual => {
                match_all_numeric_types!(compare_equal, self);
            }
            CompareNotEqual => {
                match_all_numeric_types!(compare_not_equal, self);
            }
            CompareGreater => {
                match_all_numeric_types!(compare_greater, self);
            }
            CompareGreaterEqual => {
                match_all_numeric_types!(compare_greater_equal, self);
            }
            CompareLesser => {
                match_all_numeric_types!(compare_lesser, self);
            }
            CompareLesserEqual => {
                match_all_numeric_types!(compare_lesser_equal, self);
            }

            TypeCast => {
                let type_1 = self.get::<u8>();
                let type_2 = self.get::<u8>();
                match type_1 {
                    I8 => {
                        match type_2 {
                            //I8 => self.lower_stack.cast_from_to::<i8, i8>(),
                            I16 => self.lower_stack.cast_from_to::<i8, i16>(),
                            I32 => self.lower_stack.cast_from_to::<i8, i32>(),
                            I64 => self.lower_stack.cast_from_to::<i8, i64>(),
                            U8 => self.lower_stack.cast_from_to::<i8, u8>(),
                            U16 => self.lower_stack.cast_from_to::<i8, u16>(),
                            U32 => self.lower_stack.cast_from_to::<i8, u32>(),
                            U64 => self.lower_stack.cast_from_to::<i8, u64>(),
                            F32 => self.lower_stack.cast_from_to::<i8, f32>(),
                            F64 => self.lower_stack.cast_from_to::<i8, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }

                    I16 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<i16, i8>(),
                            //I16 => self.lower_stack.cast_from_to::<i16, i16>(),
                            I32 => self.lower_stack.cast_from_to::<i16, i32>(),
                            I64 => self.lower_stack.cast_from_to::<i16, i64>(),
                            U8 => self.lower_stack.cast_from_to::<i16, u8>(),
                            U16 => self.lower_stack.cast_from_to::<i16, u16>(),
                            U32 => self.lower_stack.cast_from_to::<i16, u32>(),
                            U64 => self.lower_stack.cast_from_to::<i16, u64>(),
                            F32 => self.lower_stack.cast_from_to::<i16, f32>(),
                            F64 => self.lower_stack.cast_from_to::<i16, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    I32 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<i32, i8>(),
                            I16 => self.lower_stack.cast_from_to::<i32, i16>(),
                            //I32 => self.lower_stack.cast_from_to::<i32, i32>(),
                            I64 => self.lower_stack.cast_from_to::<i32, i64>(),
                            U8 => self.lower_stack.cast_from_to::<i32, u8>(),
                            U16 => self.lower_stack.cast_from_to::<i32, u16>(),
                            U32 => self.lower_stack.cast_from_to::<i32, u32>(),
                            U64 => self.lower_stack.cast_from_to::<i32, u64>(),
                            F32 => self.lower_stack.cast_from_to::<i32, f32>(),
                            F64 => self.lower_stack.cast_from_to::<i32, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    I64 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<i64, i8>(),
                            I16 => self.lower_stack.cast_from_to::<i64, i16>(),
                            I32 => self.lower_stack.cast_from_to::<i64, i32>(),
                            //I64 => self.lower_stack.cast_from_to::<i64, i64>(),
                            U8 => self.lower_stack.cast_from_to::<i64, u8>(),
                            U16 => self.lower_stack.cast_from_to::<i64, u16>(),
                            U32 => self.lower_stack.cast_from_to::<i64, u32>(),
                            U64 => self.lower_stack.cast_from_to::<i64, u64>(),
                            F32 => self.lower_stack.cast_from_to::<i64, f32>(),
                            F64 => self.lower_stack.cast_from_to::<i64, f64>(),

                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    U8 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<u8, i8>(),
                            I16 => self.lower_stack.cast_from_to::<u8, i16>(),
                            I32 => self.lower_stack.cast_from_to::<u8, i32>(),
                            I64 => self.lower_stack.cast_from_to::<u8, i64>(),
                            //U8 =>  self.lower_stack.cast_from_to::<u8, u8>(),
                            U16 => self.lower_stack.cast_from_to::<u8, u16>(),
                            U32 => self.lower_stack.cast_from_to::<u8, u32>(),
                            U64 => self.lower_stack.cast_from_to::<u8, u64>(),
                            F32 => self.lower_stack.cast_from_to::<u8, f32>(),
                            F64 => self.lower_stack.cast_from_to::<u8, f64>(),

                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    U16 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<u16, i8>(),
                            I16 => self.lower_stack.cast_from_to::<u16, i16>(),
                            I32 => self.lower_stack.cast_from_to::<u16, i32>(),
                            I64 => self.lower_stack.cast_from_to::<u16, i64>(),
                            U8 => self.lower_stack.cast_from_to::<u16, u8>(),
                            //U16 => self.lower_stack.cast_from_to::<u16, u16>(),
                            U32 => self.lower_stack.cast_from_to::<u16, u32>(),
                            U64 => self.lower_stack.cast_from_to::<u16, u64>(),
                            F32 => self.lower_stack.cast_from_to::<u16, f32>(),
                            F64 => self.lower_stack.cast_from_to::<u16, f64>(),

                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    U32 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<u32, i8>(),
                            I16 => self.lower_stack.cast_from_to::<u32, i16>(),
                            I32 => self.lower_stack.cast_from_to::<u32, i32>(),
                            I64 => self.lower_stack.cast_from_to::<u32, i64>(),
                            U8 => self.lower_stack.cast_from_to::<u32, u8>(),
                            U16 => self.lower_stack.cast_from_to::<u32, u16>(),
                            //U32 => self.lower_stack.cast_from_to::<u32, u32>(),
                            U64 => self.lower_stack.cast_from_to::<u32, u64>(),
                            F32 => self.lower_stack.cast_from_to::<u32, f32>(),
                            F64 => self.lower_stack.cast_from_to::<u32, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    U64 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<u64, i8>(),
                            I16 => self.lower_stack.cast_from_to::<u64, i16>(),
                            I32 => self.lower_stack.cast_from_to::<u64, i32>(),
                            I64 => self.lower_stack.cast_from_to::<u64, i64>(),
                            U8 => self.lower_stack.cast_from_to::<u64, u8>(),
                            U16 => self.lower_stack.cast_from_to::<u64, u16>(),
                            U32 => self.lower_stack.cast_from_to::<u64, u32>(),
                            //U64 => self.lower_stack.cast_from_to::<u64, u64>(),
                            F32 => self.lower_stack.cast_from_to::<u64, f32>(),
                            F64 => self.lower_stack.cast_from_to::<u64, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    F32 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<f32, i8>(),
                            I16 => self.lower_stack.cast_from_to::<f32, i16>(),
                            I32 => self.lower_stack.cast_from_to::<f32, i32>(),
                            I64 => self.lower_stack.cast_from_to::<f32, i64>(),
                            U8 => self.lower_stack.cast_from_to::<f32, u8>(),
                            U16 => self.lower_stack.cast_from_to::<f32, u16>(),
                            U32 => self.lower_stack.cast_from_to::<f32, u32>(),
                            U64 => self.lower_stack.cast_from_to::<f32, u64>(),
                            //F32 => self.lower_stack.cast_from_to::<f32, f32>(),
                            F64 => self.lower_stack.cast_from_to::<f32, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    F64 => {
                        match type_2 {
                            I8 => self.lower_stack.cast_from_to::<f64, i8>(),
                            I16 => self.lower_stack.cast_from_to::<f64, i16>(),
                            I32 => self.lower_stack.cast_from_to::<f64, i32>(),
                            I64 => self.lower_stack.cast_from_to::<f64, i64>(),
                            U8 => self.lower_stack.cast_from_to::<f64, u8>(),
                            U16 => self.lower_stack.cast_from_to::<f64, u16>(),
                            U32 => self.lower_stack.cast_from_to::<f64, u32>(),
                            U64 => self.lower_stack.cast_from_to::<f64, u64>(),
                            F32 => self.lower_stack.cast_from_to::<f64, f32>(),
                            //F64 => self.lower_stack.cast_from_to::<f64, f64>(),
                            _ => panic!("Invalid type cast!"),
                        }
                    }
                    _ => {
                        panic!("Unknown type!")
                    }
                }
            }
            _ => {
                panic!("Unknown Token! {}", Token)
            }
        }
    }
    fn execute_all(&mut self) -> () {
        let size = self.token_byte_sequence.len();
        unsafe {
            let end_ptr = self.token_byte_sequence.as_mut_ptr().add(size);

            while end_ptr.offset_from(self.cursor) > 0 {
                self.do_Token();
            }
        }
    }
}
trait StackMachineUpper: StackMachine {
    fn goto(&mut self, row_id: usize) -> ();
    fn goto_if_pop_true(&mut self, row_id: usize) -> ();
    fn goto_if_peek_true(&mut self, row_id: usize) -> ();
}
#[derive(Debug)]
enum Token {
    Push = 0,
    Pop,
    Peek,
    ClonePush,

    Add,
    Subtract,
    Multiply,
    Divide,

    Store,
    PeekStore,
    Load,

    Goto,
    PopGotoIfTrue,
    PeekGotoIfTrue,

    LogicAnd,
    LogicOr,
    LogicNot,

    CompareEqual,
    CompareNotEqual,
    CompareGreater,
    CompareGreaterEqual,
    CompareLesser,
    CompareLesserEqual,

    TypeCast,

    Bool,

    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    F32,
    F64,
}

fn tests() -> () {
    test1();
    test2();
    test3();
    test_logic_and();
    test_logic_or();
    test_logic_not();
}

fn test1() -> () {
    let mut stack = StackArray::new();
    stack.init();
    stack.push::<i32>(12);
    stack.push::<i32>(2);
    stack.push::<i32>(3);
    stack.push::<i32>(4);
    stack.multiply::<i32>();
    stack.push::<i32>(10);
    stack.push::<i32>(5);
    stack.divide::<i32>();
    stack.add::<i32>();
    stack.multiply::<i32>();
    stack.add::<u32>();

    assert_eq!(stack.pop::<i32>(), 40);
    assert_eq!(stack.end, stack.stack.as_mut_ptr());

    println!("Test 1 passed");
}

fn test2() -> () {
    let mut stack = StackArray::new();
    stack.init();
    stack.push(1i32);
    stack.push::<i32>(2);
    stack.push::<i32>(3);
    stack.push::<i32>(4);
    stack.push::<i32>(5);
    stack.push::<i32>(6);
    stack.push::<i32>(7);
    stack.push::<i32>(8);
    stack.push::<i32>(9);
    stack.push::<i32>(20);
    stack.push::<i32>(10);

    stack.divide::<i32>();
    stack.add::<i32>();
    stack.subtract::<i32>();
    stack.multiply::<i32>();
    stack.multiply::<i32>();
    stack.subtract::<i32>();
    stack.add::<i32>();
    stack.subtract::<i32>();
    stack.add::<i32>();
    stack.add::<i32>();

    //1 2 3 4 5 6 7 8 9 10 + - * * - + - + +
    assert_eq!(stack.pop::<i32>(), -129);
    assert_eq!(stack.end, stack.stack.as_mut_ptr());

    println!("Test 2 passed");
}
fn test3() -> () {
    let mut stack = StackArray::new();
    stack.init();
    stack.push::<i32>(12);
    stack.push::<i32>(2);
    stack.push::<i32>(3);
    stack.push::<i32>(4);
    stack.multiply::<i32>();
    let value = -10;
    stack.push::<u32>(value as u32);
    stack.push::<i32>(5);
    stack.divide::<i32>();
    stack.add::<i32>();
    stack.multiply::<i32>();
    stack.add::<u32>();
    assert_eq!(stack.pop::<i32>(), 32);
    assert_eq!(stack.end, stack.stack.as_mut_ptr());

    println!("Test 3 passed");
}
fn test_logic_and() -> () {
    let mut stack = StackArray::new();
    stack.init();
    stack.push(false);
    stack.push(false);
    stack.logic_and();
    assert_eq!(stack.pop::<bool>(), false);

    stack.push(false);
    stack.push(true);
    stack.logic_and();
    assert_eq!(stack.pop::<bool>(), false);

    stack.push(true);
    stack.push(false);
    stack.logic_and();
    assert_eq!(stack.pop::<bool>(), false);

    stack.push(true);
    stack.push(true);
    stack.logic_and();
    assert_eq!(stack.pop::<bool>(), true);
    assert_eq!(stack.end, stack.stack.as_mut_ptr());

    println!("Test logic and passed");
}
fn test_logic_or() -> () {
    let mut stack = StackArray::new();
    stack.init();
    stack.push(false);
    stack.push(false);
    stack.logic_or();
    assert_eq!(stack.pop::<bool>(), false);

    stack.push(false);
    stack.push(true);
    stack.logic_or();
    assert_eq!(stack.pop::<bool>(), true);

    stack.push(true);
    stack.push(false);
    stack.logic_or();
    assert_eq!(stack.pop::<bool>(), true);

    stack.push(true);
    stack.push(true);
    stack.logic_or();
    assert_eq!(stack.pop::<bool>(), true);
    assert_eq!(stack.end, stack.stack.as_mut_ptr());

    println!("Test logic or passed");
}
fn test_logic_not() -> () {
    let mut stack = StackArray::new();
    stack.init();

    stack.push(false);
    stack.logic_not();
    assert_eq!(stack.pop::<bool>(), true);

    stack.push(true);
    stack.logic_not();
    assert_eq!(stack.pop::<bool>(), false);
    assert_eq!(stack.end, stack.stack.as_mut_ptr());

    println!("Test logic not passed");
}
