llvm_canonicalize_cmake_booleans(
        MLIR_ENABLE_BINDINGS_PYTHON
)

configure_lit_site_cfg(
        ${CMAKE_CURRENT_SOURCE_DIR}/lit.site.cfg.py.in
        ${CMAKE_CURRENT_BINARY_DIR}/lit.site.cfg.py
        MAIN_CONFIG
        ${CMAKE_CURRENT_SOURCE_DIR}/lit.cfg.py
)

set(BRIL_TEST_DEPENDS
        FileCheck count not
        bril-opt
        )
# if(MLIR_ENABLE_BINDINGS_PYTHON)
#   list(APPEND BRIL_TEST_DEPENDS BrilPythonModules)
# endif()

add_lit_testsuite(check-bril "Running the bril regression tests"
        ${CMAKE_CURRENT_BINARY_DIR}
        DEPENDS ${BRIL_TEST_DEPENDS}
        )
set_target_properties(check-bril PROPERTIES FOLDER "Tests")

add_lit_testsuites(BRIL ${CMAKE_CURRENT_SOURCE_DIR} DEPENDS ${BRIL_TEST_DEPENDS})

# add_subdirectory(CAPI)
