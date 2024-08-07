add_mlir_dialect(BrilOps bril)
add_mlir_doc(BrilDialect BrilDialect Bril/ -gen-dialect-doc)
add_mlir_doc(BrilOps BrilOps Bril/ -gen-op-doc)

set(LLVM_TARGET_DEFINITIONS BrilPasses.td)
mlir_tablegen(BrilPasses.h.inc --gen-pass-decls)
add_public_tablegen_target(MLIRBrilPassesIncGen)
