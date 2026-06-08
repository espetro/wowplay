const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Native zig-glue library (for future macOS-side C interop)
    const glue_mod = b.createModule(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    const lib = b.addLibrary(.{
        .name = "wow-zig-glue",
        .linkage = .static,
        .root_module = glue_mod,
    });
    b.installArtifact(lib);

    // winerosetta.dll — cross-compiled Windows x86 DLL
    // Handles x87 instruction emulation inside Wine/CrossOver for WoW 3.3.5a
    const winerosetta_target = b.resolveTargetQuery(.{
        .cpu_arch = .x86,
        .os_tag = .windows,
        .abi = .gnu,
    });

    const winerosetta_mod = b.createModule(.{
        .target = winerosetta_target,
        .optimize = .ReleaseSafe,
        .link_libc = true,
    });
    winerosetta_mod.addCSourceFile(.{
        .file = b.path("vendor/winerosetta/winerosetta.cpp"),
        .flags = &.{
            "-std=c++17",
            "-fno-exceptions",
            "-fno-unwind-tables",
            "-fno-rtti",
        },
    });
    // SHGetKnownFolderPath requires shell32; CoTaskMemFree requires ole32
    winerosetta_mod.linkSystemLibrary("shell32", .{});
    winerosetta_mod.linkSystemLibrary("ole32", .{});

    const winerosetta = b.addLibrary(.{
        .name = "winerosetta",
        .linkage = .dynamic,
        .root_module = winerosetta_mod,
    });
    b.installArtifact(winerosetta);

    // Zig fmt check step (used by pre-commit hook)
    const fmt_check = b.addFmt(.{
        .paths = &.{"build.zig"},
        .check = true,
    });
    b.step("fmt-check", "Check Zig formatting").dependOn(&fmt_check.step);
}
