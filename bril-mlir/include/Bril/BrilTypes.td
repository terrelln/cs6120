//===- BrilTypes.td - Bril dialect types -------*- tablegen -*-===//
//
// This file is licensed under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef BRIL_TYPES
#define BRIL_TYPES

include "mlir/IR/AttrTypeBase.td"
include "Bril/BrilDialect.td"

//===----------------------------------------------------------------------===//
// Bril type definitions
//===----------------------------------------------------------------------===//

class Bril_Type<string name, string typeMnemonic, list<Trait> traits = []>
    : TypeDef<Bril_Dialect, name, traits> {
  let mnemonic = typeMnemonic;
}

def Bril_CustomType : Bril_Type<"Custom", "custom"> {
    let summary = "Bril custom type";
    let description = "Custom type in bril dialect";
    let parameters = (ins StringRefParameter<"the custom value">:$value);
    let assemblyFormat = "`<` $value `>`";
}

def Bril_IntegerType : Bril_Type<"Integer", "int"> {
    let summary = "Bril 64-bit integer type";
    let description = [{
      This type represents a 64-bit integer in the Bril language.

      It is effectively a memref<i64> in that it stores a value that can be mutated.
    }];
}

def Bril_BooleanType : Bril_Type<"Boolean", "bool"> {
    let summary = "Bril bool type";
    let description = [{
      This type represents a boolean in the Bril language.

      It is effectively a memref<bool> in that it stores a value that can be mutated.
    }];
}

def Bril_AnyType : AnyTypeOf<[Bril_IntegerType, Bril_BooleanType]>;

#endif // BRIL_TYPES
