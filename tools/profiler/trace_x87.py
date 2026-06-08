#!/usr/bin/env python3
"""
x87 call tracer using Frida.

Attaches to the Wine process running WoW.exe and traces x87 math
calls via Frida hooks on msvcrt.dll exports.
"""

import argparse
import json
import os
import signal
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import frida

from attach import find_wow_process, load_config


def load_frida_script(script_path: Path) -> str:
    """Load a Frida script from file."""
    with open(script_path) as f:
        return f.read()


def on_message(message: dict, data: bytes, trace_buffer: list):
    """Handle messages from Frida script."""
    if message["type"] == "send":
        trace_buffer.append(message["payload"])
    elif message["type"] == "error":
        print(f"Frida error: {message['description']}", file=sys.stderr)


def run_trace(
    pid: int,
    script_source: str,
    duration: int,
    output_dir: Path,
) -> Path:
    """
    Run x87 trace on the given PID for the specified duration.
    
    Args:
        pid: Process ID to attach to
        script_source: Frida script source code
        duration: Tracing duration in seconds
        output_dir: Directory to save trace output
        
    Returns:
        Path to saved trace file
    """
    trace_buffer = []
    session = None
    
    try:
        # Attach to process
        print(f"Attaching to PID {pid}...")
        session = frida.attach(pid)
        
        # Create script
        script = session.create_script(script_source)
        script.on("message", lambda msg, data: on_message(msg, data, trace_buffer))
        
        print("Loading script...")
        script.load()
        
        # Set up signal handler for graceful shutdown
        def signal_handler(sig, frame):
            print("\nReceived signal, saving partial data...")
            raise KeyboardInterrupt
        
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        # Run for specified duration
        print(f"Tracing x87 calls for {duration} seconds...")
        print("Press Ctrl+C to stop early and save partial data")
        
        start_time = time.time()
        try:
            while time.time() - start_time < duration:
                time.sleep(1)
                elapsed = int(time.time() - start_time)
                if elapsed % 30 == 0 and elapsed > 0:
                    print(f"  ... {elapsed}s elapsed, {len(trace_buffer)} calls captured")
        except KeyboardInterrupt:
            print("\nStopping trace early...")
        
        print(f"Trace complete. Captured {len(trace_buffer)} x87 calls.")
        
    except frida.ProcessNotFoundError:
        raise RuntimeError(f"Process {pid} not found")
    except frida.PermissionDeniedError:
        raise RuntimeError(
            f"Permission denied attaching to {pid}. "
            "This may be due to SIP/hardened runtime. "
            "Try using --fallback mode or disable SIP for this process."
        )
    except frida.ServerNotRunningError:
        raise RuntimeError(
            "Frida server not running. "
            "Install frida-server or use frida-tools."
        )
    finally:
        if session:
            try:
                session.detach()
                print("Detached from process")
            except Exception as e:
                print(f"Error detaching: {e}", file=sys.stderr)
    
    # Save trace data
    output_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    output_file = output_dir / f"raw_trace_{timestamp}.json"
    
    trace_data = {
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "pid": pid,
        "duration_seconds": int(time.time() - start_time),
        "call_count": len(trace_buffer),
        "calls": trace_buffer,
    }
    
    with open(output_file, "w") as f:
        json.dump(trace_data, f, indent=2)
    
    print(f"Trace saved to: {output_file}")
    return output_file


def main():
    parser = argparse.ArgumentParser(
        description="Trace x87 math calls in WoW via Frida"
    )
    parser.add_argument(
        "--duration", "-d",
        type=int,
        default=300,
        help="Tracing duration in seconds (default: 300)",
    )
    parser.add_argument(
        "--output-dir", "-o",
        type=Path,
        help="Output directory for trace data",
    )
    parser.add_argument(
        "--script", "-s",
        type=Path,
        help="Path to custom Frida script",
    )
    parser.add_argument(
        "--fallback",
        action="store_true",
        help="Use sample-based fallback instead of Frida",
    )
    
    args = parser.parse_args()
    
    try:
        config = load_config()
        
        # Validate duration
        min_dur = config["profiling"].get("min_duration", 10)
        max_dur = config["profiling"].get("max_duration", 3600)
        if args.duration < min_dur:
            print(f"Duration too short (minimum {min_dur}s)", file=sys.stderr)
            return 1
        if args.duration > max_dur:
            print(f"Duration too long (maximum {max_dur}s)", file=sys.stderr)
            return 1
        
        # Find WoW process
        pid = find_wow_process(config)
        
        # Determine output directory
        output_dir = args.output_dir
        if not output_dir:
            output_dir = Path(config["profiling"].get("output_dir", "data/profiling"))
        output_dir = output_dir.resolve()
        
        if args.fallback:
            print("Using sample-based fallback mode")
            # Import here to avoid dependency if not used
            from sample_cpu import run_sample, parse_sample_output, save_samples
            
            raw_output = run_sample(pid, args.duration)
            parsed = parse_sample_output(raw_output)
            parsed["pid"] = pid
            parsed["duration_seconds"] = args.duration
            
            output_file = save_samples(parsed, output_dir)
            print(f"Fallback samples saved to: {output_file}")
            return 0
        
        # Load Frida script
        if args.script:
            script_path = args.script
        else:
            script_path = Path(__file__).parent / "frida_scripts" / "x87_hook.js"
        
        if not script_path.exists():
            print(f"Frida script not found: {script_path}", file=sys.stderr)
            return 1
        
        script_source = load_frida_script(script_path)
        
        # Run trace
        output_file = run_trace(pid, script_source, args.duration, output_dir)
        return 0
        
    except RuntimeError as e:
        print(f"Error: {e}", file=sys.stderr)
        if "SIP" in str(e) or "Permission denied" in str(e):
            print("\nTip: Try running with --fallback to use sample-based profiling", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
