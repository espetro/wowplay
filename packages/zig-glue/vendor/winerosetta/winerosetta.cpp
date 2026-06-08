// Vendored from https://github.com/Gcenx/winerosetta
// Modified: export Direct3DCreate9 directly via dllexport (no .def file needed for Zig cross-build)
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <d3d9.h>
#include <shlobj.h>

static void log_to_file(const char* msg) {
    FILE* f = fopen("C:/winerosetta_debug.log", "a");
    if (f) {
        fprintf(f, "%s\n", msg);
        fclose(f);
    }
}

LONG WINAPI
VectoredHandler1(
    struct _EXCEPTION_POINTERS *ExceptionInfo
    )
{
    if (ExceptionInfo->ExceptionRecord->ExceptionCode == EXCEPTION_ILLEGAL_INSTRUCTION) {
        auto context = ExceptionInfo->ContextRecord;
        char buffer[256];
        uint16_t instr = *reinterpret_cast<uint16_t*>(context->Eip);
        sprintf(buffer, "[VEH] ILLEGAL_INSTRUCTION at EIP=%08X, instr=%04X", context->Eip, instr);
        log_to_file(buffer);

        if (instr == 0xD063) {
            log_to_file("[VEH] -> Matched ARPL (0xD063), emulating");
            // emulate arpl ax, dx
            auto dest = reinterpret_cast<uint16_t*>(&context->Eax);
            auto src = reinterpret_cast<uint16_t*>(&context->Edx);
            if ((*dest & 3) < (*src & 3)) {
                context->EFlags |= 0x40; // set ZF
                *dest = (*dest & ~3) | (*src & 3);
            } else {
                context->EFlags &= ~0x40; // clear ZF
            }
            context->Eip += 2;
            return EXCEPTION_CONTINUE_EXECUTION;
        }

        // fcomp st -> fcomp st0 fixup (WoW 3.3.5a specific)
        if (instr == 0xD8DC) {
            log_to_file("[VEH] -> Matched FCOMP (0xD8DC), patching");
            DWORD oldProtect;
            VirtualProtect(reinterpret_cast<void*>(context->Eip), 2, PAGE_EXECUTE_READWRITE, &oldProtect);
            *reinterpret_cast<uint16_t*>(context->Eip) = 0xD8D8; // fcomp st0
            VirtualProtect(reinterpret_cast<void*>(context->Eip), 2, PAGE_EXECUTE_READ, &oldProtect);
            return EXCEPTION_CONTINUE_EXECUTION;
        }

        sprintf(buffer, "[VEH] -> No match for instr=%04X at %08X", instr, context->Eip);
        log_to_file(buffer);
    }
    return EXCEPTION_CONTINUE_SEARCH;
}

BOOL WINAPI DllMain(HMODULE module, DWORD reason, LPVOID reserved) {
    if (reason == DLL_PROCESS_ATTACH) {
        DisableThreadLibraryCalls(module);
        TCHAR module_path[MAX_PATH];
        if (GetModuleFileName(module, module_path, MAX_PATH)) {
            LoadLibrary(module_path);
        }
        AddVectoredExceptionHandler(1, VectoredHandler1);
    }
    return TRUE;
}

typedef IDirect3D9* (WINAPI *Direct3DCreate9_t)(UINT);

static Direct3DCreate9_t real_Direct3DCreate9 = nullptr;

static HMODULE load_d3d9(void) {
    HMODULE d3d9 = LoadLibrary("d9vk.dll");
    if (!d3d9) {
        PWSTR system32_path = nullptr;
        HRESULT hr = SHGetKnownFolderPath(FOLDERID_System, 0, NULL, &system32_path);
        if (SUCCEEDED(hr)) {
            char d3d9_path[MAX_PATH];
            sprintf(d3d9_path, "%S\\d3d9.dll", system32_path);
            d3d9 = LoadLibraryEx(d3d9_path, 0, LOAD_WITH_ALTERED_SEARCH_PATH);
            CoTaskMemFree(system32_path);
        }
    }
    return d3d9;
}

// Direct export via dllexport: proxies Direct3DCreate9 to d9vk.dll or system d3d9.dll
extern "C" __declspec(dllexport) IDirect3D9* WINAPI Direct3DCreate9(UINT SDKVersion) {
    if (!real_Direct3DCreate9) {
        HMODULE d3d9 = load_d3d9();
        if (d3d9) {
            real_Direct3DCreate9 = (Direct3DCreate9_t)GetProcAddress(d3d9, "Direct3DCreate9");
        }
    }
    if (!real_Direct3DCreate9) return nullptr;
    return real_Direct3DCreate9(SDKVersion);
}
