/**
 * Frida script to trace D3D9 draw calls.
 * 
 * Hooks key D3D9 device methods to identify rendering hotspots
 * and correlate them with x87 usage patterns.
 */

// D3D9 functions to hook
var d3d9Functions = [
    "DrawPrimitive",
    "DrawIndexedPrimitive", 
    "SetStreamSource",
    "SetTransform",
];

// Frame tracking
var frameCount = 0;
var frameStartTime = Date.now();
var callsThisFrame = {};

// Reset frame counters
function resetFrame() {
    frameCount++;
    var now = Date.now();
    var frameTime = now - frameStartTime;
    
    // Send frame summary every frame
    var summary = {
        type: "d3d9_frame",
        frame: frameCount,
        frame_time_ms: frameTime,
        calls: callsThisFrame,
    };
    send(summary);
    
    // Reset for next frame
    callsThisFrame = {};
    frameStartTime = now;
}

// Hook a D3D9 function
function hookD3D9Function(funcName) {
    var export = Module.findExportByName("d3d9.dll", funcName);
    if (!export) {
        console.log("[!] D3D9 export not found: " + funcName);
        return;
    }
    
    console.log("[+] Hooking d3d9.dll!" + funcName + " at " + export);
    
    Interceptor.attach(export, {
        onEnter: function(args) {
            // Count calls per frame
            if (!callsThisFrame[funcName]) {
                callsThisFrame[funcName] = 0;
            }
            callsThisFrame[funcName]++;
        }
    });
}

// Main
console.log("[*] D3D9 hook script loaded");

// Install hooks
d3d9Functions.forEach(function(func) {
    hookD3D9Function(func);
});

// Frame tracking via Present (end of frame marker)
var present = Module.findExportByName("d3d9.dll", "Present");
if (present) {
    console.log("[+] Hooking d3d9.dll!Present at " + present);
    Interceptor.attach(present, {
        onEnter: function(args) {
            resetFrame();
        }
    });
}

console.log("[*] D3D9 hooks installed");
