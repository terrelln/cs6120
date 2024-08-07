get_property(dialect_libs GLOBAL PROPERTY MLIR_DIALECT_LIBS)
get_property(conversion_libs GLOBAL PROPERTY MLIR_CONVERSION_LIBS)
set(LIBS
        ${dialect_libs}
        ${conversion_libs}
        MLIRArithDialect
        MLIROptLib
        MLIRBril
        )
add_llvm_executable(bril-opt bril-opt.cpp)

llvm_update_compile_flags(bril-opt)
target_link_libraries(bril-opt PRIVATE ${LIBS})

mlir_check_all_link_libraries(bril-opt)
