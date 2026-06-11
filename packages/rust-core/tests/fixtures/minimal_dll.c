// Compile with: zig cc -target i386-windows-gnu -shared -o minimal.dll minimal_dll.c

__declspec(dllexport) int add(int a, int b) {
    return a + b;
}
