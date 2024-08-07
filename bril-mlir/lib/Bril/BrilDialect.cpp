//===- BrilDialect.cpp - Bril dialect ---------------*- C++ -*-===//
//
// This file is licensed under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#include "Bril/BrilDialect.h"
#include "Bril/BrilOps.h"
#include "Bril/BrilTypes.h"

using namespace mlir;
using namespace mlir::bril;

#include "Bril/BrilOpsDialect.cpp.inc"

//===----------------------------------------------------------------------===//
// Bril dialect.
//===----------------------------------------------------------------------===//

void BrilDialect::initialize() {
  addOperations<
#define GET_OP_LIST
#include "Bril/BrilOps.cpp.inc"
      >();
  registerTypes();
}
