# Synacor CTF virtual machine implementation

A cutting-edge VM for the Synacor CTF, meticulously crafted in Rust.

Embark on the Synacor Capture the Flag challenge armed with only the "arch-spec" and "challenge.bin" files, and triumph with our VM implementation based on the architecture file. Our VM offers an unparalleled competitive edge, complete with debugging capabilities through an intuitive CLI escape mechanism.

Key features include:

 - Debugger CLI with its own micro language
 - Efficient pausing and resuming of the VM without transmitting keystrokes
 - Memory viewing and editing for registers and stack
 - Step-by-step execution with detailed assembly display, including opcodes, descriptions, and parameters
 - Snapshot creation and recovery for seamless progress tracking
 - Fully cross platform

Coming soon:
 - Advanced breakpoint options for program points, specific operations, and register access
 - Parsing and execution separation for improved efficiency
 - Implementation of debugger ABI OPs
 - API integration for independent debugger processes
 - Debugger configuration
 - Debugger as seperate framework
 - A VM harness for any future VMs
 - Using macros for rapid OPcode definition
 - Debug print separation from operation execution
 - GUI or TUI implementation for real-time visualization of stack and registers during code execution
 - Dynamic optimization for faster code execution
 - JIT compilation integration for enhanced performance
 - Real-time code analysis and visualization for easier debugging and optimization
 - Customizable plugin system to extend VM functionality
