add_mlir_dialect_library(MLIRBril
        BrilTypes.cpp
        BrilDialect.cpp
        BrilOps.cpp
        BrilPasses.cpp

        ADDITIONAL_HEADER_DIRS
        ${PROJECT_SOURCE_DIR}/include/Bril

        DEPENDS
        MLIRBrilOpsIncGen
        MLIRBrilPassesIncGen

        LINK_LIBS PUBLIC
        MLIRIR
        MLIRInferTypeOpInterface
        MLIRFuncDialect
        )
