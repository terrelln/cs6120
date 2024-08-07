cmake_minimum_required(VERSION 3.20.0)
project(bril-mlir LANGUAGES CXX C)

set(CMAKE_BUILD_WITH_INSTALL_NAME_DIR ON)

set(CMAKE_CXX_STANDARD 17 CACHE STRING "C++ standard to conform to")

# Build via external projects mechanism
set(MLIR_MAIN_SRC_DIR ${LLVM_MAIN_SRC_DIR}/../mlir)
set(MLIR_INCLUDE_DIR ${MLIR_MAIN_SRC_DIR}/include)
set(MLIR_GENERATED_INCLUDE_DIR ${LLVM_BINARY_DIR}/tools/mlir/include)
set(MLIR_INCLUDE_DIRS "${MLIR_INCLUDE_DIR};${MLIR_GENERATED_INCLUDE_DIR}")

# if(MLIR_ENABLE_BINDINGS_PYTHON)
#   include(MLIRDetectPythonEnv)
#   mlir_configure_python_dev_packages()
# endif()

set(BRIL_SOURCE_DIR ${PROJECT_SOURCE_DIR})
set(BRIL_BINARY_DIR ${PROJECT_BINARY_DIR})
include_directories(${LLVM_INCLUDE_DIRS})
include_directories(${MLIR_INCLUDE_DIRS})
include_directories(${BRIL_SOURCE_DIR}/include)
include_directories(${BRIL_BINARY_DIR}/include)
link_directories(${LLVM_BUILD_LIBRARY_DIR})
add_definitions(${LLVM_DEFINITIONS})

add_subdirectory(include)
add_subdirectory(lib)
if(MLIR_ENABLE_BINDINGS_PYTHON)
  message(STATUS "Enabling Python API")
  add_subdirectory(python)
endif()
add_subdirectory(test)
add_subdirectory(bril-opt)
# add_subdirectory(standalone-plugin)
# add_subdirectory(standalone-translate)