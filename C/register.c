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

int lgsys_JIT_R_NUM(void) {
    return JIT_R_NUM;
}

int lgsys_JIT_V_NUM(void) {
    return JIT_V_NUM;
}

int lgsys_JIT_F_NUM(void) {
    return JIT_F_NUM;
}
