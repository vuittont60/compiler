mod linker;

use intrusive_collections::RBTree;

pub use self::linker::{Linker, LinkerError};

use super::*;

/// A [Program] is a collection of [Module]s that are being compiled together as a package.
///
/// This is primarily used for storing/querying data which must be shared across modules:
///
/// * The set of global variables which will be allocated on the global heap
/// * The set of modules and functions which have been defined
///
/// When translating to Miden Assembly, we need something like this to allow us to perform some
/// basic linker tasks prior to emitting the textual MASM which will be fed to the Miden VM.
///
/// This structure is intended to be allocated via [std::sync::Arc], so that it can be shared
/// across multiple threads which are emitting/compiling modules at the same time. It is designed
/// so that individual fields are locked, rather than the structure as a whole, to minimize contention.
/// The intuition is that, in general, changes at the [Program] level are relatively infrequent, i.e.
/// only when declaring a new [Module], or [GlobalVariable], do we actually need to mutate the structure.
/// In all other situations, changes are scoped at the [Module] level.
pub struct Program {
    /// This tree stores all of the modules being compiled as part of the current program.
    modules: RBTree<ModuleTreeAdapter>,
    /// If set, this field indicates which function is the entrypoint for the program.
    ///
    /// When generating Miden Assembly, this will determine whether or not we're emitting
    /// a program or just a collection of modules; and in the case of the former, what code
    /// to emit in the root code block.
    entrypoint: Option<FunctionIdent>,
    /// The data segments gathered from all modules in the program, and laid out in address order.
    segments: DataSegmentTable,
    /// The global variable table produced by linking the global variable tables of all
    /// modules in this program. The layout of this table corresponds to the layout of
    /// global variables in the linear memory heap at runtime.
    globals: GlobalVariableTable,
}
impl Default for Program {
    /// A default [Program] is equivalent to calling [Program::new], i.e. it
    /// is preloaded with builtin globals and functions.
    fn default() -> Self {
        Self::new()
    }
}
impl Program {
    /// Create a new, empty [Program].
    ///
    /// NOTE: An empty program will be missing some builtin global variables
    /// and functions that are expected to be linked when generating code for
    /// the Miden VM. It is expected that these will be provided manually, or
    /// that the generated code does not rely on them.
    #[inline]
    pub fn empty() -> Self {
        Self {
            modules: Default::default(),
            entrypoint: None,
            segments: Default::default(),
            globals: Default::default(),
        }
    }

    /// Create a new [Program] with the set of builtin globals and functions
    /// needed to support some key functionality on the Miden VM.
    pub fn new() -> Self {
        // TODO: Flesh this out with the kernel
        Self::empty()
    }

    /// Returns true if this program has a defined entrypoint
    pub const fn has_entrypoint(&self) -> bool {
        self.entrypoint.is_none()
    }

    /// Returns true if this program is executable.
    ///
    /// An executable program is one which has an entrypoint that will be called
    /// after the program is loaded.
    pub const fn is_executable(&self) -> bool {
        self.has_entrypoint()
    }

    /// Returns the [Function] representing the entrypoint of this program
    pub fn entrypoint(&self) -> Option<&Function> {
        let id = self.entrypoint?;
        let module = self
            .modules
            .find(&id.module)
            .get()
            .expect("invalid entrypoint: unknown module");
        let entry = module.function(id.function);
        debug_assert!(entry.is_some(), "invalid entrypoint: unknown function");
        entry
    }

    /// Return a reference to the data segment table for this program
    pub fn segments(&self) -> &DataSegmentTable {
        &self.segments
    }

    /// Get a reference to the global variable table for this program
    pub fn globals(&self) -> &GlobalVariableTable {
        &self.globals
    }

    /// Get a mutable reference to the global variable table for this program
    pub fn globals_mut(&mut self) -> &mut GlobalVariableTable {
        &mut self.globals
    }

    /// Returns true if `name` is defined in this program.
    pub fn contains(&self, name: Ident) -> bool {
        !self.modules.find(&name).is_null()
    }
}
