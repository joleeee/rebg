#include "pin.H"
#include <vector>
#include <string>
#include <utility>
#include <stdint.h>
#include <assert.h>

using UTIL::REGVALUE;
using std::vector;

FILE* out;

inline VOID printReg(CONTEXT *context, REG reg) {
    std::string name = REG_StringShort(reg);

    UINT32 regsize = REG_Size(reg);
    uint8_t buf[8];
    assert(regsize == 8);

    PIN_GetContextRegval(context, reg, (uint8_t*)buf);
    fprintf(out, "|%s=%lx", name.c_str(), *(uint64_t*)buf);
}

VOID dumpRegs(CONTEXT *context) {
    fprintf(out, "regs");

    printReg(context, REG_RIP);
    for(REG reg = REG_RDI; reg <= REG_R15; reg++) {
        printReg(context, reg);
    }

    fprintf(out, "\n");
}

VOID instrumentInstruction(INS ins, VOID *v) {
    ADDRINT address = INS_Address(ins);
    USIZE size = INS_Size(ins);

    fprintf(out, "step|adr=%lx|code=", address);

    for(USIZE i = 0; i < size; i++) {
        uint8_t * a = (uint8_t*)address + i;
        fprintf(out, "%02x", *a);
    }
    fprintf(out, "\n");

    // quite expensive
    INS_InsertCall(ins, IPOINT_BEFORE, (AFUNPTR)dumpRegs, IARG_CONTEXT, IARG_END);
}

VOID fini(INT32 code, VOID *v) {
    fclose(out);
}

int main(int argc, char *argv[]) {
    if (PIN_Init(argc, argv)) {
        return 1;
    }
    out = fopen("/tmp/rebg-pin", "w");
    INS_AddInstrumentFunction(instrumentInstruction, 0);
    PIN_AddFiniFunction(fini, 0);
    PIN_StartProgram();
    return 0;
}
