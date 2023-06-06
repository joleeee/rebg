#include "pin.H"
#include <vector>
#include <string>
#include <utility>
#include <stdint.h>

FILE* out;

VOID instrumentInstruction(INS ins, VOID *v) {
    ADDRINT address = INS_Address(ins);
    USIZE size = INS_Size(ins);

    fprintf(out, "step|adr=%lx|code=", address);

    for(USIZE i = 0; i < size; i++) {
        uint8_t * a = (uint8_t*)address + i;
        fprintf(out, "%02x", *a);
    }
    fprintf(out, "\n");
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
