// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/console.sol";
import {Push3Interpreter} from "../src/Push3Interpreter.sol";

/**
 * @title Push3IntAdapter
 * @dev An adapter contract for our Push3Interpreter that provides
 *      various integer-only I/O function overloads. We now create
 *      a SUBLIST descriptor for the entire code array, placing it
 *      on the EXEC stack. We ignore finalCodeStack/finalExecStack 
 *      to avoid compiler warnings.
 */
contract Push3IntAdapter {
    Push3Interpreter public interpreter;

    constructor(address _interpreter) {
        interpreter = Push3Interpreter(_interpreter);
    }

    /**
     * @notice Utility function to build a single sublist descriptor
     *         covering the entire `code` array, offset=0, length=code.length.
     */
    function _makeFullSublistDescriptor(bytes calldata code)
        internal
        view
        returns (uint256)
    {
        // CodeTag.SUBLIST is typically enum = 4, but
        // let's fetch it via
        //   (uint8)Push3Interpreter.CodeTag.SUBLIST
        // or we can directly pass 3 if we know the numeric value.

        // We'll do:
        return interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST, // the tag
            0,                                // offset
            uint32(code.length),             // length of entire code array
            0                                 // leftover bits=0
        );
    }

    /**
     * @notice Single int input, single int output.
     *  1) Build a sublist descriptor for the entire code,
     *  2) Place that descriptor in the EXEC stack,
     *  3) Preload `in1` on the integer stack,
     *  4) Run the interpreter,
     *  5) Return top of final int stack.
     */
    function run1In1Out(bytes calldata code, int256 in1)
        external
        view
        returns (int256 out1)
    {
        // 1) Build the sublist descriptor
        uint256 fullDesc = _makeFullSublistDescriptor(code);

        // 2) EXEC stack of size 1
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = fullDesc;

        // No code stack
        uint256[] memory initCodeStack = new uint256[](0);

        // 3) Preload int stack
        int256[] memory initIntStack = new int256[](1);
        initIntStack[0] = in1;

        // 4) Run
        (, , int256[] memory finalIntStack,) =
            interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        // 5) Return top of final int stack
        require(finalIntStack.length > 0, "No integer output produced");
        out1 = finalIntStack[finalIntStack.length - 1];
    }

    /**
     * @notice Two ints input, single int output
     */
    function run2In1Out(bytes calldata code, int256 in1, int256 in2)
        external
        view
        returns (int256 out1)
    {
        uint256 fullDesc = _makeFullSublistDescriptor(code);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = fullDesc;

        uint256[] memory initCodeStack = new uint256[](0);

        int256[] memory initIntStack = new int256[](2);
        initIntStack[0] = in1;
        initIntStack[1] = in2;

        (, , int256[] memory finalIntStack,) =
            interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        require(finalIntStack.length > 0, "No integer output");

        out1 = finalIntStack[finalIntStack.length - 1];
    }

    /**
     * @notice Single int input, two int outputs
     */
    function run1In2Out(bytes calldata code, int256 in1)
        external
        view
        returns (int256 out1, int256 out2)
    {
        uint256 fullDesc = _makeFullSublistDescriptor(code);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = fullDesc;

        uint256[] memory initCodeStack = new uint256[](0);

        int256[] memory initIntStack = new int256[](1);
        initIntStack[0] = in1;

        (, , int256[] memory finalIntStack,) =
            interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        require(finalIntStack.length >= 2, "Not enough outputs");
        out1 = finalIntStack[finalIntStack.length - 1];
        out2 = finalIntStack[finalIntStack.length - 2];
    }

    /**
     * @notice Zero int input, returns entire final int stack
     */
    function run0InNOut(bytes calldata code)
        external
        view
        returns (int256[] memory)
    {
        uint256 fullDesc = _makeFullSublistDescriptor(code);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = fullDesc;

        uint256[] memory initCodeStack = new uint256[](0);
        int256[] memory initIntStack = new int256[](0);

        (, , int256[] memory finalIntStack,) =
            interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        return finalIntStack;
    }
}
