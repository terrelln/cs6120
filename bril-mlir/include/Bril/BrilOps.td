//===- BrilOps.td - Bril dialect ops -----------*- tablegen -*-===//
//
// This file is licensed under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//

#ifndef BRIL_OPS
#define BRIL_OPS

include "Bril/BrilTypes.td"
include "mlir/Interfaces/InferTypeOpInterface.td"
include "mlir/Interfaces/SideEffectInterfaces.td"

def Bril_FooOp : Bril_Op<"foo", [Pure,
                                             SameOperandsAndResultType]> {
    let summary = "Illustrates how to define an operation.";
    let description = [{
        The `bril.foo` operation illustrates how to define a new
        operation in a dialect. It uses an operation trait to declare that it
        has no side effects.

        This operation takes an integer argument and returns an integer.

        Example:

        ```mlir
        %0 = arith.constant 2 : i32
        // Apply the foo operation to %0
        %1 = bril.foo %0 : i32
        ```
    }];

    let arguments = (ins I32:$input);
    let results = (outs I32:$res);

    let assemblyFormat = [{
        $input attr-dict `:` type($input)
    }];
}

def DeclareOp : Bril_Op<"declare", [Pure]> {
    let summary = "Declare a variable";
    let description = [{
        This operation declares a variable in the Bril language.
    }];

    let arguments = (ins);

    let results = (outs Bril_AnyType:$result);

    let assemblyFormat = [{
        attr-dict `:` type($result)
    }];
}

#endif // BRIL_OPS
