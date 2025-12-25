#!/usr/bin/env python3
"""
Generate a synthetic C project for benchmarking ninja vs rninja.

Usage:
    python3 gen_bench.py [num_files] [output_dir]

Arguments:
    num_files   - Number of C source files to generate (default: 100)
    output_dir  - Directory to create the project in (default: current directory)

This creates:
    - N source files (file_0.c, file_1.c, ...)
    - Corresponding headers with declarations
    - A main.c that includes all headers
    - A build.ninja file to compile everything
"""

import os
import sys
import argparse


def generate_header(i: int) -> str:
    """Generate a header file with function declaration."""
    return f"""#ifndef FILE_{i}_H
#define FILE_{i}_H

int compute_{i}(int x);

#endif
"""


def generate_source(i: int, num_files: int) -> str:
    """Generate a source file with some computation."""
    # Include some headers to create dependencies
    includes = [f'#include "file_{j}.h"' for j in range(max(0, i-3), i)]
    includes_str = "\n".join(includes) if includes else ""

    return f"""#include "file_{i}.h"
{includes_str}

int compute_{i}(int x) {{
    int result = x * {i + 1};
    // Add some bulk to make compilation take time
    for (int j = 0; j < 100; j++) {{
        result += j * {i % 10 + 1};
    }}
    return result;
}}
"""


def generate_main(num_files: int) -> str:
    """Generate main.c that uses all the functions."""
    includes = [f'#include "file_{i}.h"' for i in range(num_files)]
    calls = [f"    result += compute_{i}(i);" for i in range(num_files)]

    return f"""#include <stdio.h>
{chr(10).join(includes)}

int main() {{
    int result = 0;
    for (int i = 0; i < 1000; i++) {{
{chr(10).join(calls)}
    }}
    printf("Result: %d\\n", result);
    return 0;
}}
"""


def generate_ninja(num_files: int) -> str:
    """Generate build.ninja file."""
    lines = [
        "# Auto-generated build.ninja for benchmarking",
        "",
        "cc = gcc",
        "cflags = -O2 -Wall",
        "",
        "rule cc",
        "    command = $cc $cflags -c $in -o $out",
        "    description = CC $out",
        "",
        "rule link",
        "    command = $cc $in -o $out",
        "    description = LINK $out",
        "",
    ]

    # Build rules for each source file
    objects = []
    for i in range(num_files):
        obj = f"file_{i}.o"
        src = f"file_{i}.c"
        hdr = f"file_{i}.h"
        # Include dependency headers
        deps = [f"file_{j}.h" for j in range(max(0, i-3), i)]
        deps_str = " ".join([hdr] + deps)
        lines.append(f"build {obj}: cc {src} | {deps_str}")
        objects.append(obj)

    # Main object
    main_deps = " ".join([f"file_{i}.h" for i in range(num_files)])
    lines.append(f"build main.o: cc main.c | {main_deps}")
    objects.append("main.o")

    # Link
    objects_str = " ".join(objects)
    lines.append(f"")
    lines.append(f"build program: link {objects_str}")
    lines.append(f"")
    lines.append(f"default program")
    lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Generate a synthetic C project for benchmarking"
    )
    parser.add_argument(
        "num_files",
        type=int,
        nargs="?",
        default=100,
        help="Number of C source files to generate (default: 100)"
    )
    parser.add_argument(
        "output_dir",
        type=str,
        nargs="?",
        default=".",
        help="Directory to create the project in (default: current directory)"
    )

    args = parser.parse_args()
    num_files = args.num_files
    output_dir = args.output_dir

    # Create output directory if needed
    os.makedirs(output_dir, exist_ok=True)

    print(f"Generating benchmark project with {num_files} files in {output_dir}/")

    # Generate source and header files
    for i in range(num_files):
        header_path = os.path.join(output_dir, f"file_{i}.h")
        source_path = os.path.join(output_dir, f"file_{i}.c")

        with open(header_path, "w") as f:
            f.write(generate_header(i))

        with open(source_path, "w") as f:
            f.write(generate_source(i, num_files))

    # Generate main.c
    main_path = os.path.join(output_dir, "main.c")
    with open(main_path, "w") as f:
        f.write(generate_main(num_files))

    # Generate build.ninja
    ninja_path = os.path.join(output_dir, "build.ninja")
    with open(ninja_path, "w") as f:
        f.write(generate_ninja(num_files))

    print(f"Generated:")
    print(f"  - {num_files} source files (file_0.c ... file_{num_files-1}.c)")
    print(f"  - {num_files} header files (file_0.h ... file_{num_files-1}.h)")
    print(f"  - main.c")
    print(f"  - build.ninja")
    print()
    print("To benchmark:")
    print(f"  cd {output_dir}")
    print("  # Clean build with ninja")
    print("  rm -f *.o program .ninja_log && time ninja")
    print("  # Clean build with rninja")
    print("  rm -f *.o program .ninja_log && time rninja")


if __name__ == "__main__":
    main()
