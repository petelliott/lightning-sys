#include <lightning.h>
#include "lightning-sys.h"

int lgsys_jit_r(int n) {
    return JIT_R(n);
}

int lgsys_jit_v(int n) {
    return JIT_V(n);
}

int lgsys_jit_f(int n) {
    return JIT_F(n);
}

const int lgsys_JIT_R_NUM = JIT_R_NUM;
const int lgsys_JIT_V_NUM = JIT_V_NUM;
const int lgsys_JIT_F_NUM = JIT_F_NUM;
