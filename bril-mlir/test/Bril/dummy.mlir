// RUN: bril-opt %s | bril-opt | FileCheck %s

module {
    // CHECK-LABEL: func @bar()
    func.func @bar() {
        %0 = arith.constant 1 : i32
        // CHECK: %{{.*}} = bril.foo %{{.*}} : i32
        %res = bril.foo %0 : i32
        return
    }

    // CHECK-LABEL: func @bril_types(%arg0: !bril.custom<"10">)
    func.func @bril_types(%arg0: !bril.custom<"10">) {
        return
    }
}
