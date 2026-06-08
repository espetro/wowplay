/**
 * Frida script to hook x87 math functions in msvcrt.dll.
 * 
 * Hooks common math exports and captures:
 * - Function name
 * - Return address (to identify caller)
 * - Timestamp
 * - Module information
 * 
 * Filters by WoW.exe address range to distinguish Wine x87 from WoW x87.
 */

// Configuration - will be injected by Python runner
var config = {
    wowBase: ptr("0x00400000"),
    wowEnd: ptr("0x00800000"),
    filterWine: true,
};

// x87 functions to hook in msvcrt.dll
var x87Functions = [
    "sin",
    "cos", 
    "tan",
    "sqrt",
    "pow",
    "floor",
    "ceil",
    "fmod",
    "_ftol",
    "_ftol2",
    "atan2",
    "log",
    "log10",
    "exp",
];

// Check if address is within WoW.exe range
function isWowAddress(addr) {
    if (!config.filterWine) {
        return true;
    }
    return addr.compare(config.wowBase) >= 0 && addr.compare(config.wowEnd) <= 0;
}

// Send trace entry
function sendTrace(funcName, retAddr, moduleName) {
    var entry = {
        type: "x87_call",
        func: funcName,
        ret_addr: retAddr.toString(),
        module: moduleName,
        timestamp: Date.now(),
    };
    send(entry);
}

// Hook a single function
function hookFunction(moduleName, funcName) {
    var module = Process.findModuleByName(moduleName);
    if (!module) {
        console.log("[!] Module not found: " + moduleName);
        return;
    }
    
    var export = Module.findExportByName(moduleName, funcName);
    if (!export) {
        console.log("[!] Export not found: " + moduleName + "!" + funcName);
        return;
    }
    
    console.log("[+] Hooking " + moduleName + "!" + funcName + " at " + export);
    
    Interceptor.attach(export, {
        onLeave: function(retval) {
            var retAddr = this.returnAddress;
            
            // Filter by WoW address range
            if (config.filterWine && !isWowAddress(retAddr)) {
                return;
            }
            
            sendTrace(funcName, retAddr, moduleName);
        }
    });
}

// Main
console.log("[*] x87 hook script loaded");
console.log("[*] WoW address range: " + config.wowBase + " - " + config.wowEnd);

// Hook all x87 functions
var modules = ["msvcrt.dll", "msvcrt40.dll"];
modules.forEach(function(module) {
    x87Functions.forEach(function(func) {
        hookFunction(module, func);
    });
});

console.log("[*] x87 hooks installed");
