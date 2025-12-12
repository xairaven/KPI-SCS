# Software of Computer Systems

This repository contains the laboratory coursework for the "Software of Computer Systems" discipline at Igor Sikorsky Kyiv Polytechnic Institute. The project focuses on the software implementation of compiler functions, including lexical and syntactic analysis, as well as algorithms for the optimal parallelization of computations for computer systems with specific architectures.

The entire project is implemented using the **Rust** programming language, ensuring memory safety and high performance for the computational tasks involved.

## Repository Structure

The codebase is organized into directories corresponding to the progression of the laboratory assignments.

### Lab 1: Lexical and Syntactic Analysis
Located in the `Lab1` directory, this module implements the initial stage of a compiler. Its primary goal is to perform lexical and syntactic analysis of a given arithmetic expression. The program accepts input strings containing algebraic operations, variable names, constants, and parentheses. It validates the expression against the grammar rules, checking for errors such as mismatched parentheses, invalid operator usage, or incorrect syntax. The output is a validation report indicating whether the expression is correct or listing specific syntax errors found.

### Lab 2: Automatic Parallelization
The `Lab2` directory contains the implementation for constructing a parallel form of the arithmetic expression. Using the validated expression from the first lab, this module builds a parallel computation tree designed to maximize the "width" (number of operations performed simultaneously) and minimize the "height" (number of execution steps). The algorithm analyzes data dependencies to determine which operations can be executed concurrently, creating a model for a parallel execution schedule.

### Lab 3-4: Equivalent Forms Generation
These combined laboratories, found in the `Lab3-4` directory, expand on the parallelization concepts by applying algebraic laws. The objective is to generate a set of equivalent forms for the initial arithmetic expression. By applying rules such as commutativity, distributivity, associativity, and bracket expansion, the software explores different structural variations of the expression. This allows the system to find alternative parallel forms that might be more efficient for specific hardware configurations.

### Lab 5-6: Execution Modeling and Optimization
The `Lab5-6` directory addresses the modeling and optimization of execution on specific Parallel Computer Systems (PCS). This module takes the parallel trees generated in previous steps and simulates their execution on a defined architecture (e.g., a system with a specific number of processors or a pipeline). It calculates key performance metrics including execution time, speedup coefficients, and system efficiency. The final component of this work involves selecting the optimal parallel form from the set of equivalent forms generated earlier, identifying the variation that yields the best performance for the target hardware.

## Usage

To run any of the laboratory works, ensure you have the Rust toolchain installed. Navigate to the specific directory (e.g., `cd Lab1`) and execute the project using `cargo run`. Some directories may contain shell scripts like `start.sh` or `tests.sh` to facilitate running the application or its test suite.

## License

This project is licensed under the terms specified in the `LICENSE` file located in the root directory.