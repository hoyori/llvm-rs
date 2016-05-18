use libc::{c_char, c_uint, c_int};
use ffi::prelude::*;
use ffi::{core, LLVMAttribute};
use ffi::LLVMLinkage;
use std::ffi::CString;
use std::{fmt, mem};
use std::ops::{Deref, Index};
use block::BasicBlock;
use context::{Context, GetContext};
use ty::{FunctionType, Type};
use util::{self, CastFrom};

/// A typed value that can be used as an operand in instructions.
pub struct Value;
native_ref!(&Value = LLVMValueRef);
impl Value {
    /// Create a new constant struct from the values given.
    pub fn new_struct<'a>(context: &'a Context, vals: &[&'a Value], packed: bool) -> &'a Value {
        unsafe { core::LLVMConstStructInContext(context.into(), vals.as_ptr() as *mut LLVMValueRef, vals.len() as c_uint, packed as c_int) }.into()
    }
    /// Create a new constant vector from the values given.
    pub fn new_vector<'a>(vals: &[&'a Value]) -> &'a Value {
        unsafe { core::LLVMConstVector(vals.as_ptr() as *mut LLVMValueRef, vals.len() as c_uint).into() }
    }
    /// Create a new constant C string from the text given.
    pub fn new_string<'a>(context: &'a Context, text: &str, rust_style: bool) -> &'a Value {
        unsafe {
            let ptr = text.as_ptr() as *const c_char;
            let len = text.len() as c_uint;
            core::LLVMConstStringInContext(context.into(), ptr, len, rust_style as c_int).into()
        }
    }
    /// Create a new constant undefined value of the given type.
    pub fn new_undef<'a>(ty: &'a Type) -> &'a Value {
        unsafe { core::LLVMGetUndef(ty.into()).into() }
    }
    /// Returns the name of this value, or `None` if it lacks a name
    pub fn get_name(&self) -> Option<&str> {
        unsafe {
            let c_name = core::LLVMGetValueName(self.into());
            util::to_null_str(c_name as *mut i8)
        }
    }
    /// Sets the name of this value
    pub fn set_name(&self, name: &str) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            core::LLVMSetValueName(self.into(), c_name.as_ptr())
        }
    }
    /// Returns the type of this value
    pub fn get_type(&self) -> &Type {
        unsafe { core::LLVMTypeOf(self.into()) }.into()
    }
}
/// Comparative operations on values.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Predicate {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual
}
/// A function argument.
pub struct Arg;
native_ref!(&Arg = LLVMValueRef);
impl Deref for Arg {
    type Target = Value;
    fn deref(&self) -> &Value {
        unsafe { mem::transmute(self) }
    }
}
impl Arg {
    /// Add the attribute given to this argument.
    pub fn add_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMAddAttribute(self.into(), attr.into()) }
    }
    /// Add all the attributes given to this argument.
    pub fn add_attributes(&self, attrs: &[Attribute]) {
        let mut sum = LLVMAttribute::empty();
        for attr in attrs {
            let attr:LLVMAttribute = (*attr).into();
            sum = sum | attr;
        }
        unsafe { core::LLVMAddAttribute(self.into(), sum.into()) }
    }
    /// Returns true if this argument has the attribute given.
    pub fn has_attribute(&self, attr: Attribute) -> bool {
        unsafe {
            let other = core::LLVMGetAttribute(self.into());
            other.contains(attr.into())
        }
    }
    /// Returns true if this argument has all the attributes given.
    pub fn has_attributes(&self, attrs: &[Attribute]) -> bool {
        unsafe {
            let other = core::LLVMGetAttribute(self.into());
            for &attr in attrs {
                if !other.contains(attr.into()) {
                    return false;
                }
            }
            return true;
        }
    }
    /// Remove an attribute from this argument.
    pub fn remove_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMRemoveAttribute(self.into(), attr.into()) }
    }
}

/// A global value (eg: Function, Alias, Global variable)
pub struct GlobalValue;
native_ref!(&GlobalValue = LLVMValueRef);
impl Deref for GlobalValue {
    type Target = Value;
    fn deref(&self) -> &Value {
        unsafe { mem::transmute(self) }
    }
}
impl CastFrom for GlobalValue {
    type From = Value;
    fn cast<'a>(val: &'a Value) -> Option<&'a GlobalValue> {
        unsafe {
            let global = mem::transmute(core::LLVMIsAGlobalValue(val.into()));
            util::ptr_to_null(global)
        }
    }
}
impl GlobalValue {
    /// Set the linkage type for this global
    pub fn set_linkage(&self, linkage: Linkage) {
        unsafe {
            core::LLVMSetLinkage(self.into(), linkage.into());
        }
    }
    /// Returns the linkage type for this global
    pub fn get_linkage(&self) -> Linkage {
        unsafe {
            core::LLVMGetLinkage(self.into()).into()
        }
    }
    /// Returns true if this global is a declaration (as opposed to a definition).
    pub fn is_declaration(&self) -> bool {
        unsafe {
            // FIXME: There should be a constant somewhere, instead of '1'
            core::LLVMIsDeclaration(self.into()) == 1
        }
    }
}

/// A global variable
pub struct GlobalVariable;
native_ref!(&GlobalVariable = LLVMValueRef);
impl Deref for GlobalVariable {
    type Target = GlobalValue;
    fn deref(&self) -> &GlobalValue {
        unsafe { mem::transmute(self) }
    }
}
impl CastFrom for GlobalVariable {
    type From = Value;
    fn cast<'a>(val: &'a Value) -> Option<&'a GlobalVariable> {
        unsafe {
            let global = mem::transmute(core::LLVMIsAGlobalVariable(val.into()));
            util::ptr_to_null(global)
        }
    }
}
impl GlobalVariable {
    /// Set the initial value of the global
    pub fn set_initializer(&self, val: &Value) {
        unsafe {
            core::LLVMSetInitializer(self.into(), val.into())
        }
    }
    /// Set the initial value of the global
    pub fn get_initializer(&self) -> Option<&Value> {
        unsafe {
            util::ptr_to_null(core::LLVMGetInitializer(self.into()))
        }
    }
    /// Set whether this global is a constant.
    pub fn set_constant(&self, is_constant: bool) {
        let llvm_bool = if is_constant { 1 } else { 0 };
        unsafe {
            core::LLVMSetGlobalConstant(self.into(), llvm_bool);
        }
    }
    /// Returns true if this global is a constant.
    pub fn get_constant(&self) -> bool {
        unsafe {
            // FIXME: There should be a constant for True/False
            core::LLVMIsGlobalConstant(self.into()) == 1
        }
    }
}

/// An alias to another global
pub struct Alias;
native_ref!(&Alias = LLVMValueRef);
impl Deref for Alias {
    type Target = GlobalValue;
    fn deref(&self) -> &GlobalValue {
        unsafe { mem::transmute(self) }
    }
}
impl CastFrom for Alias {
    type From = Value;
    fn cast<'a>(val: &'a Value) -> Option<&'a Alias> {
        unsafe {
            let alias = mem::transmute(core::LLVMIsAGlobalAlias(val.into()));
            util::ptr_to_null(alias)
        }
    }
}

/// A function that can be called and contains blocks.
pub struct Function;
native_ref!(&Function = LLVMValueRef);
impl Deref for Function {
    type Target = GlobalValue;
    fn deref(&self) -> &GlobalValue {
        unsafe { mem::transmute(self) }
    }
}
impl Index<usize> for Function {
    type Output = Arg;
    fn index(&self, index: usize) -> &Arg {
        unsafe {
            if index < core::LLVMCountParams(self.into()) as usize {
                core::LLVMGetParam(self.into(), index as c_uint).into()
            } else {
                panic!("no such index {} on {:?}", index, self.get_type())
            }
        }
    }
}
impl CastFrom for Function {
    type From = Value;
    fn cast<'a>(val: &'a Value) -> Option<&'a Function> {
        let ty = val.get_type();
        let mut is_func = ty.is_function();
        if let Some(elem) = ty.get_element() {
            is_func = is_func || elem.is_function()
        }
        if is_func {
            Some(unsafe { mem::transmute(val) })
        } else {
            None
        }
    }
}
impl Function {
    /// Add a basic block with the name given to the function and return it.
    pub fn append<'a>(&'a self, name: &str) -> &'a BasicBlock {
        util::with_cstr(name, |ptr| unsafe {
            core::LLVMAppendBasicBlockInContext(self.get_context().into(), self.into(), ptr).into()
        })
    }
    /// Returns the entry block of this function or `None` if there is none.
    pub fn get_entry(&self) -> Option<&BasicBlock> {
        unsafe { mem::transmute(core::LLVMGetEntryBasicBlock(self.into())) }
    }
    /// Returns the name of this function.
    pub fn get_name(&self) -> &str {
        unsafe {
            let c_name = core::LLVMGetValueName(self.into());
            util::to_str(c_name as *mut i8)
        }
    }
    /// Returns the function signature representing this function's signature.
    pub fn get_signature(&self) -> &FunctionType {
        unsafe {
            let ty = core::LLVMTypeOf(self.into());
            core::LLVMGetElementType(ty).into()
        }
    }
    /// Add the attribute given to this function.
    pub fn add_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMAddFunctionAttr(self.into(), attr.into()) }
    }
    /// Add all the attributes given to this function.
    pub fn add_attributes(&self, attrs: &[Attribute]) {
        let mut sum = LLVMAttribute::empty();
        for attr in attrs {
            let attr:LLVMAttribute = (*attr).into();
            sum = sum | attr;
        }
        unsafe { core::LLVMAddFunctionAttr(self.into(), sum.into()) }
    }
    /// Returns true if the attribute given is set in this function.
    pub fn has_attribute(&self, attr: Attribute) -> bool {
        unsafe {
            let other = core::LLVMGetFunctionAttr(self.into());
            other.contains(attr.into())
        }
    }
    /// Returns true if all the attributes given is set in this function.
    pub fn has_attributes(&self, attrs: &[Attribute]) -> bool {
        unsafe {
            let other = core::LLVMGetFunctionAttr(self.into());
            for &attr in attrs {
                if !other.contains(attr.into()) {
                    return false;
                }
            }
            return true;
        }
    }
    /// Remove the attribute given from this function.
    pub fn remove_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMRemoveFunctionAttr(self.into(), attr.into()) }
    }
}
impl GetContext for Function {
    fn get_context(&self) -> &Context {
        self.get_type().get_context()
    }
}

pub struct PhiNode;
native_ref!(&PhiNode = LLVMValueRef);
impl Deref for PhiNode {
    type Target = Value;
    fn deref(&self) -> &Value {
        unsafe { mem::transmute(self) }
    }
}
impl PhiNode {
    pub fn add_incoming(&self, in_coming_val: &Value, in_coming_bb:&BasicBlock) {
        unsafe { core::LLVMAddIncoming(self.into(), (&[in_coming_val]).as_ptr() as *mut LLVMValueRef, 
            (&[in_coming_bb]).as_ptr() as *mut LLVMBasicBlockRef, 1usize as c_uint)}
    }
}

/// A way of indicating to LLVM how you want arguments / functions to be handled.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub enum Attribute {
    /// Zero-extended before or after call.
    ZExt =              0b1,
    /// Sign-extended before or after call.
    SExt =              0b10,
    /// Mark the function as not returning.
    NoReturn =          0b100,
    /// Force argument to be passed in register.
    InReg =             0b1000,
    /// Hidden pointer to structure to return.
    StructRet =         0b10000,
    /// Function doesn't unwind stack.
    NoUnwind =          0b100000,
    /// Consider to not alias after call.
    NoAlias =           0b1000000,
    /// Pass structure by value.
    ByVal =             0b10000000,
    /// Nested function static chain.
    Nest =              0b100000000,
    /// Function doesn't access memory.
    ReadNone =          0b1000000000,
    /// Function only reads from memory.
    ReadOnly =          0b10000000000,
    /// Never inline this function.
    NoInline =          0b100000000000,
    /// Always inline this function.
    AlwaysInline =      0b1000000000000,
    /// Optimize this function for size.
    OptimizeForSize =   0b10000000000000,
    /// Stack protection.
    StackProtect =      0b100000000000000,
    /// Stack protection required.
    StackProtectReq =   0b1000000000000000,
    /// Alignment of parameter (5 bits) stored as log2 of alignment with +1 bias 0 means unaligned (different from align(1)).
    Alignment =         0b10000000000000000,
    /// Function creates no aliases of pointer.
    NoCapture =         0b100000000000000000,
    /// Disable redzone.
    NoRedZone =         0b1000000000000000000,
    /// Disable implicit float instructions.
    NoImplicitFloat =   0b10000000000000000000,
    /// Naked function.
    Naked =             0b100000000000000000000,
    /// The source language has marked this function as inline.
    InlineHint =        0b1000000000000000000000,
    /// Alignment of stack for function (3 bits) stored as log2 of alignment with +1 bias 0 means unaligned (different from alignstack=(1)).
    StackAlignment =    0b11100000000000000000000000000,
    /// This function returns twice.
    ReturnsTwice =      0b100000000000000000000000000000,
    /// Function must be in unwind table.
    UWTable =           0b1000000000000000000000000000000,
    /// Function is called early/often, so lazy binding isn't effective.
    NonLazyBind =       0b10000000000000000000000000000000
}
impl From<LLVMAttribute> for Attribute {
    fn from(attr: LLVMAttribute) -> Attribute {
        unsafe { mem::transmute(attr) }
    }
}
impl From<Attribute> for LLVMAttribute {
    fn from(attr: Attribute) -> LLVMAttribute {
        unsafe { mem::transmute(attr) }
    }
}

/// A way of indicating to LLVM how you want a global to interact during linkage.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub enum Linkage {
    /// Default linkage. The global is externally visible and participates in linkage normally.
    External            = 0,
    /// Never emitted to the containing module's object file. Used to allow inlining and/or other optimisations to take place, given knowledge of the definition of the global, which is somewhere outside of the module. Otherwise the same as LinkOnceODR. Only allowed on definitions, not declarations.
    AvailableExternally = 1,
    /// Merged with other globals of the same name during linkage. Unreferenced LinkOnce globals may be discarded.
    LinkOnceAny         = 2,
    /// Similar to LinkOnceAny, but indicates that it will only be merged with equivalent globals.
    LinkOnceODR         = 3,
    /// Same merging semantics as LinkOnceAny. Unlike LinkOnce, unreference globals will not be discarded.
    WeakAny             = 5,
    /// Similar to WeakAny, but indicates that it will only be merged with equivalent globals.
    WeakODR             = 6,
    /// Only allowed on global array pointers. When two globals with Appending linkage are merged, they are appended together.
    Appending           = 7,
    /// Similar to Private, but shows as a local symbol in the object file.
    Internal            = 8,
    /// Only directly accessible by objects in the current module. May be renamed as neccessary to avoid collisions, and all references will be updated. Will not show up in the object file's symbol table.
    Private             = 9,
    /// Weak until linked. If not linked, the output symbol is null, instead of undefined.
    ExternalWeak        = 12,
    /// Similar to Weak, but may not have an explicit section, must have a zero initializer, and may not be marked constant. Cannot be used on functions or aliases.
    Common              = 14,
}
impl From<LLVMLinkage> for Linkage {
    fn from(attr: LLVMLinkage) -> Linkage {
        unsafe { mem::transmute(attr) }
    }
}
impl From<Linkage> for LLVMLinkage {
    fn from(attr: Linkage) -> LLVMLinkage {
        unsafe { mem::transmute(attr) }
    }
}

impl GetContext for Value {
    fn get_context(&self) -> &Context {
        self.get_type().get_context()
    }
}
to_str!(Value, LLVMPrintValueToString);

