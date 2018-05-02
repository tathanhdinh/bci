// extern crate inkwell;
// #![feature(proc_macro_path_invoc)]
// #![feature(use_extern_macros)]

#[macro_use] extern crate structopt;
#[macro_use] extern crate failure;
extern crate llvm_sys as llvm;
extern crate dot;
use structopt::StructOpt;

use std::io::{BufReader, Read};
// use std::borrow::IntoCow;
// use std::convert::Into;

#[derive(StructOpt, Debug)]
#[structopt(name = "LLVM bitcode inspector")]
struct Opt {
    #[structopt(help = "input bitcode file")]
    file: String,

    #[structopt(short = "f", long = "function", help = "function name")]
    function: String
}

// impl<'a> dot::Labeller<'a, 
type Nd = llvm::prelude::LLVMBasicBlockRef;
type Ed = (Nd, Nd);
struct Graph {
    func: llvm::prelude::LLVMValueRef,
    basic_blocks: Vec<Nd>
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        let graph_name = unsafe {
            let name = llvm::core::LLVMGetValueName(self.func);
            std::ffi::CStr::from_ptr(name)
        };
        dot::Id::new(graph_name.to_str().unwrap()).unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        let basic_block_name = unsafe {
            // let value_name = llvm::core::LLVMGetValueName(n as &llvm::prelude::LLVMValueRef);
            let value = llvm::core::LLVMBasicBlockAsValue(*n);
            let name = llvm::core::LLVMGetValueName(value);
            std::ffi::CStr::from_ptr(name)
        };
        dot::Id::new(basic_block_name.to_str().unwrap()).unwrap()
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a, Nd> {
        let mut basic_blocks = Vec::new();
        let mut bb = {
            llvm::core::LLVMGetFirstBasicBlock(self.func)
        };
        while bb != std::ptr::null_mut() {
            basic_blocks.push(bb);
            bb = unsafe {
                llvm::core::LLVMGetNextBasicBlock(bb)
            };
            basic_blocks.push(bb);
        }
        std::borrow::Cow::Owned(basic_blocks)
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {

    }
}

fn run() -> Result<(), failure::Error> {
    let opt = Opt::from_args();

    // let bc_filename = opt.bitcode_file.ok_or_else(|| {format_err!("input bitcode file is not given")})?;
    let bc_filename = opt.file;
    let bc_file = std::fs::File::open(&bc_filename)?;

    let mut bc_buffer = Vec::new();
    let mut bc_reader = std::io::BufReader::new(bc_file);
    bc_reader.read_to_end(&mut bc_buffer)?;

    let mb = unsafe {
        let bc_filename = std::ffi::CString::new(bc_filename.clone().trim_right_matches(".bc"))?;
        llvm::core::LLVMCreateMemoryBufferWithMemoryRangeCopy(bc_buffer.as_ptr() as *const i8, bc_buffer.len(), bc_filename.as_ptr())
    };

    let context = unsafe {
        llvm::core::LLVMContextCreate()
    };

    let mut module = std::ptr::null_mut();
    let mut out_msg = std::ptr::null_mut();
    let parse_bc_ret = unsafe {
        llvm::bit_reader::LLVMParseBitcodeInContext(context, mb, &mut module, &mut out_msg)
    };
    if parse_bc_ret != 0 {
        return Err(format_err!("cannot parse input bitcode (code {})", parse_bc_ret));
    }

    // let mut func = unsafe {
    //     llvm::core::LLVMGetFirstFunction(module)
    // };

    // while func != std::ptr::null_mut() {
    //     let func_name = unsafe { llvm::core::LLVMGetValueName(func) };
    //     if func_name != std::ptr::null() {
    //         let func_name = unsafe { std::ffi::CStr::from_ptr(func_name) };
    //         println!("{}", func_name.to_str()?);
    //     }
    //     func = unsafe { llvm::core::LLVMGetNextFunction(func) };
    // }

    let func_name = opt.function;
    let func_name = std::ffi::CString::new(func_name)?;
    let func = unsafe {
        llvm::core::LLVMGetNamedFunction(module, func_name.as_ptr())
    };

    unsafe {
        llvm::analysis::LLVMViewFunctionCFG(func);
    }

    // dispose
    unsafe {
        llvm::core::LLVMDisposeMemoryBuffer(mb);
        llvm::core::LLVMDisposeModule(module);
        llvm::core::LLVMContextDispose(context);
    }

    Ok(())
}

fn main() {
    if let Err(ref err) = run() {
        println!("Error: {}", err);
    }

    // unsafe {
    //     // let mb = llvm::LLVMMemoryBuffer;
    //     let context = llvm::core::LLVMContextCreate();
    //     // let mb = llvm::bit_reader::LLVMParseBitcode();
    // }

    // println!("Hello, world!");
    // let _ = inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default()).unwrap();
    // let context = inkwell::context::Context::create();
    // let module = context.create_module("sum");
    // let builder = context.create_builder();
    // let execution_engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None).unwrap();

    // let i64_type = context.i64_type();
    // let fn_type = i64_type.fn_type(&[&i64_type, &i64_type, &i64_type], false);

    // let function = module.add_function("sum", &fn_type, None);
    // let basic_block = context.append_basic_block(&function, "entry");

    // builder.position_at_end(&basic_block);
    // let x = function.get_nth_param(0).unwrap().into_int_value();
    // let y = function.get_nth_param(1).unwrap().into_int_value();
    // let z = function.get_nth_param(2).unwrap().into_int_value();

    // let sum = builder.build_int_add(&x, &y, "sum");
    // let sum = builder.build_int_add(&sum, &z, "sum");

    // builder.build_return(Some(&sum));

    // let sum_func = unsafe { 
    //     execution_engine.get_function::<unsafe extern "C" fn(u64, u64, u64) -> u64>("sum").unwrap()
    // };
    // println!("1 + 2 + 3 = {}", unsafe { sum_func(1, 2, 3) });

    // let addr = execution_engine.get_function_address("sum").unwrap();
    // println!("function address: 0x{:x}", addr);

    // let sum: extern "C" fn(u64, u64, u64) -> u64 = unsafe { std::mem::transmute(addr) };
    // println!("1 + 2 + 3 = {}", sum(1, 2, 3));
}
