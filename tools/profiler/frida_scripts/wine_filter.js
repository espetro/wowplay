/**
 * Wine address range filter for Frida scripts.
 * 
 * Provides helper functions to distinguish WoW.exe code from Wine code.
 * Used by x87_hook.js and d3d9_hook.js to filter traces.
 */

// Default WoW.exe memory range for 32-bit PE
// These should be updated after memory map documentation (T04)
var wowMemoryRange = {
    start: ptr("0x00400000"),
    end: ptr("0x00800000"),
};

/**
 * Check if an address is within the WoW.exe memory range.
 * 
 * @param {NativePointer} addr - Address to check
 * @returns {boolean} true if address is in WoW.exe range
 */
function isWowAddress(addr) {
    return addr.compare(wowMemoryRange.start) >= 0 && 
           addr.compare(wowMemoryRange.end) <= 0;
}

/**
 * Check if an address is within Wine's own code (not WoW).
 * 
 * @param {NativePointer} addr - Address to check  
 * @returns {boolean} true if address is in Wine range
 */
function isWineAddress(addr) {
    return !isWowAddress(addr);
}

/**
 * Get module name for an address.
 * 
 * @param {NativePointer} addr - Address to look up
 * @returns {string} Module name or "unknown"
 */
function getModuleForAddress(addr) {
    var module = Process.findModuleByAddress(addr);
    if (module) {
        return module.name;
    }
    return "unknown";
}

/**
 * Update the WoW memory range from config.
 * Called by the Python runner with actual values from config.toml.
 * 
 * @param {string} start - Hex string of start address (e.g., "0x00400000")
 * @param {string} end - Hex string of end address (e.g., "0x00800000")
 */
function setWowMemoryRange(start, end) {
    wowMemoryRange.start = ptr(start);
    wowMemoryRange.end = ptr(end);
    console.log("[*] WoW memory range updated: " + start + " - " + end);
}

// Export functions for use by other scripts
module.exports = {
    isWowAddress: isWowAddress,
    isWineAddress: isWineAddress,
    getModuleForAddress: getModuleForAddress,
    setWowMemoryRange: setWowMemoryRange,
};

console.log("[*] Wine filter loaded");
console.log("[*] Default WoW range: " + wowMemoryRange.start + " - " + wowMemoryRange.end);
